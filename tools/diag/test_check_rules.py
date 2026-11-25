"""Tests for the agent-rule check: each failure it exists for is built here
and must be reported, and a clean tree must pass."""

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import check_rules as cr  # noqa: E402

HEADER = ("- Scope: measuring.\n"
          "- Paths: `tools/`, `build.rs`, `docs/*.txt`.\n"
          "- Before: running a match.\n")
CORE = ("# Rules\n\nThe core stays within 18,000 bytes.\n\n"
        "### `docs/rules/measure.md`\n" + HEADER + "\n## Changes\n\n- Never push.\n")
MODULE = "# Measuring\n\nA module.\n\n" + HEADER + "\n## Measurement\n\n- Rebuild first.\n"
PLAN = ("# Plan\n\n## 2. Rules\n\nrules\n\n## Phase C - closed\n\nsummary\n\n"
        "### Active workflow register\n\n| Leaf | Workflow state | Class | Current decision |\n"
        "|---|---|---|---|\n| D.1 | RESEARCH | R2 | decide |\n\n## Phase D\n\nsteps\n")


def tree(**changes):
    files = {"AGENTS.md": CORE, "CLAUDE.md": "# Stub\n\n@AGENTS.md\n",
             "docs/rules/measure.md": MODULE, "docs/PLAN.md": PLAN,
             "tools/run.ps1": "", "build.rs": "", "src/lib.rs": ""}
    for path, text in changes.items():
        path = path.replace("__", "/").replace("_md", ".md")
        if text is None:
            del files[path]
        else:
            files[path] = text
    return files


class Budgets(unittest.TestCase):
    def test_a_clean_tree_passes(self):
        self.assertEqual(cr.problems(tree()), [])

    def test_a_core_grown_past_its_budget_fails(self):
        grown = CORE + "- A rule.\n" * 2000
        found = cr.problems(tree(AGENTS_md=grown))
        self.assertTrue(any("loads" in p and "over the core budget" in p for p in found), found)

    def test_the_stub_and_its_imports_are_counted_together(self):
        near = CORE + "x" * (cr.CORE_BUDGET - cr.loaded_size(CORE) - 10) + "\n"
        self.assertEqual(cr.problems(tree(AGENTS_md=near, CLAUDE_md=None)), [])
        found = cr.problems(tree(AGENTS_md=near))
        self.assertEqual(len(found), 1, found)
        self.assertIn("the set CLAUDE.md loads", found[0])

    def test_size_is_counted_with_crlf_endings(self):
        self.assertEqual(cr.loaded_size("a\nb\r\n"), 6)

    def test_a_module_past_its_budget_fails(self):
        found = cr.problems(tree(docs__rules__measure_md=MODULE + "- More.\n" * 3000))
        self.assertTrue(any("over the module budget" in p for p in found), found)

    def test_the_core_must_state_the_budget(self):
        found = cr.problems(tree(AGENTS_md=CORE.replace("18,000", "20,000")))
        self.assertTrue(any("does not state the core budget" in p for p in found), found)

    def test_the_orientation_read_has_a_budget(self):
        long_row = PLAN.replace("| decide |", "| " + "decided " * 3000 + "|")
        found = cr.problems(tree(docs__PLAN_md=long_row))
        self.assertTrue(any("orientation read" in p for p in found), found)
        steps = PLAN.replace("steps", "steps " * 4000)
        self.assertEqual(cr.problems(tree(docs__PLAN_md=steps)), [])
        found = cr.problems(tree(docs__PLAN_md=PLAN.replace("### Active", "### Idle")))
        self.assertTrue(any("to measure the orientation" in p for p in found), found)


class Index(unittest.TestCase):
    def test_a_module_missing_from_the_index_fails(self):
        found = cr.problems(tree(docs__rules__extra_md=MODULE))
        self.assertEqual(found, ["docs/rules/extra.md is not in AGENTS.md's index"])

    def test_an_index_entry_without_a_module_fails(self):
        found = cr.problems(tree(docs__rules__measure_md=None))
        self.assertEqual(found, ["AGENTS.md's index names docs/rules/measure.md, "
                                 "which does not exist"])

    def test_a_module_without_its_header_fails(self):
        bare = "# Measuring\n\n## Measurement\n\n- Rebuild first.\n"
        found = cr.problems(tree(docs__rules__measure_md=bare))
        self.assertTrue(any("lacks its header" in p for p in found), found)

    def test_a_header_that_drifted_from_the_index_fails(self):
        drifted = MODULE.replace("running a match", "running a tune")
        found = cr.problems(tree(docs__rules__measure_md=drifted))
        self.assertTrue(any("differ" in p for p in found), found)

    def test_triggers_below_the_first_section_are_not_the_header(self):
        late = "# Measuring\n\n## Measurement\n\n" + HEADER
        found = cr.problems(tree(docs__rules__measure_md=late))
        self.assertTrue(any("lacks its header" in p for p in found), found)

    def test_a_path_trigger_that_names_nothing_fails(self):
        gone = tree()
        del gone["build.rs"]
        found = cr.problems(gone)
        self.assertEqual(found, ["docs/rules/measure.md: the path trigger `build.rs` "
                                 "names nothing in the tree"])


class Imports(unittest.TestCase):
    def test_a_stub_that_imports_a_module_fails(self):
        stub = "# Stub\n\n@AGENTS.md\n@docs/rules/measure.md\n"
        found = cr.problems(tree(CLAUDE_md=stub))
        self.assertTrue(any("CLAUDE.md imports the module" in p for p in found), found)

    def test_an_include_in_the_core_is_an_import(self):
        core = CORE + "See @[measuring](docs/rules/measure.md).\n"
        found = cr.problems(tree(AGENTS_md=core))
        self.assertTrue(any("AGENTS.md imports the module" in p for p in found), found)
        self.assertTrue(any("CLAUDE.md imports the module" in p for p in found), found)

    def test_a_module_named_by_its_path_is_not_imported(self):
        files = tree()
        self.assertEqual(cr.closure("CLAUDE.md", files), ["CLAUDE.md", "AGENTS.md"])
        self.assertEqual(cr.closure("AGENTS.md", files), ["AGENTS.md"])

    def test_an_import_inside_code_or_naming_no_file_is_not_one(self):
        text = "`@docs/rules/measure.md` and @maintainer and a@AGENTS.md\n```\n@AGENTS.md\n```\n"
        self.assertEqual(cr.imports("X.md", text, tree()), [])

    def test_an_import_resolves_beside_the_importing_file(self):
        files = tree(docs__rules__other_md="x")
        self.assertEqual(cr.imports("docs/rules/measure.md", "@other.md\n", files),
                         ["docs/rules/other.md"])

    def test_a_rules_directory_and_a_nested_rule_file_join_every_set(self):
        files = tree()
        files[".claude/rules/style.md"] = "s"
        files["tools/AGENTS.md"] = "t"
        self.assertEqual(cr.auto_loaded_sets(files)["AGENTS.md"],
                         ["AGENTS.md", ".claude/rules/style.md", "tools/AGENTS.md"])


class Triggered(unittest.TestCase):
    def test_a_directory_a_file_and_a_glob_fire(self):
        index = cr.parse_index(CORE)
        changed = ["build.rs", "docs/a.txt", "src/lib.rs", "tools/x/run.ps1"]
        self.assertEqual(cr.triggered(index, changed),
                         {"docs/rules/measure.md": ["tools/x/run.ps1", "build.rs", "docs/a.txt"]})

    def test_nothing_fires_on_an_untriggered_path(self):
        self.assertEqual(cr.triggered(cr.parse_index(CORE), ["src/lib.rs", "xbuild.rs"]), {})


if __name__ == "__main__":
    unittest.main()
