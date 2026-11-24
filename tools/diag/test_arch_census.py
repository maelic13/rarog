import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("arch_census.py")
SPEC = importlib.util.spec_from_file_location("arch_census", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


def write_tree(root: Path, files: dict[str, str]) -> None:
    for rel, text in files.items():
        path = root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")


TREE = {
    "src/lib.rs": "pub mod a;\npub mod b;\n",
    "src/a.rs": (
        "use crate::b::helper;\n"
        "/// A doc comment naming `pub fn hidden` in prose.\n"
        "pub fn used() -> u32 { helper() + 1 }\n"
        "pub fn unused() {}\n"
        "pub(crate) fn only_tests() {}\n"
        "pub(crate) const LONELY: u8 = 3;\n"
        "\n"
        "#[cfg(test)]\n"
        "mod tests {\n"
        "    #[test]\n"
        "    fn t() { super::only_tests(); }\n"
        "}\n"
    ),
    "src/b.rs": (
        "use crate::a::used;\n"
        "pub fn helper() -> u32 { 2 }\n"
        "pub fn calls() -> u32 { used() }\n"
        "#[cfg(test)]\n"
        "mod tests { fn t() { crate::a::only_tests(); } }\n"
    ),
    "tests/it.rs": "fn main() { rarog::a::unused_by_test(); }\n",
}


class MaskTests(unittest.TestCase):
    def test_comments_and_literals_blanked_offsets_kept(self):
        src = 'let s = "a // b"; // tail\nlet c = \'x\'; /* x /* y */ z */ let r = r#"q"#;\n'
        masked = MODULE.mask(src)
        self.assertEqual(len(masked), len(src))
        self.assertEqual(masked.count("\n"), src.count("\n"))
        self.assertNotIn("tail", masked)
        self.assertNotIn("// b", masked)
        self.assertNotIn("z */", masked)
        self.assertIn("let r = r#", masked)
        self.assertNotIn("q", masked.split("let r")[1])

    def test_lifetime_is_not_a_char_literal(self):
        src = "fn f<'a>(x: &'a str) -> &'a str { x }\n"
        self.assertEqual(MODULE.mask(src), src)

    def test_cfg_test_region_covers_the_module(self):
        src = TREE["src/a.rs"]
        regs = MODULE.cfg_test_regions(MODULE.mask(src))
        self.assertEqual(len(regs), 1)
        start, end = regs[0]
        self.assertTrue(src[start:].startswith("#[cfg(test)]"))
        self.assertEqual(src[end], "}")
        self.assertTrue(src[:end + 1].endswith("super::only_tests(); }\n}"))

    def test_line_classes(self):
        src = "// c\n\nfn f() {}\n#[cfg(test)]\nmod t {}\n"
        masked = MODULE.mask(src)
        cls = MODULE.classify_lines(src, masked, MODULE.cfg_test_regions(masked))
        self.assertEqual(cls, ["comment", "blank", "code", "test", "test", "blank"])


class TreeTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        write_tree(self.root, TREE)
        self.census = MODULE.census(self.root)

    def tearDown(self):
        self.tmp.cleanup()

    def item(self, name):
        found = [it for it in self.census["public"]["detail"] if it["name"] == name]
        self.assertEqual(len(found), 1, name)
        return found[0]

    def test_public_census_classes(self):
        self.assertEqual(self.item("used")["class"], "used")
        self.assertEqual(self.item("unused")["class"], "zero")
        self.assertEqual(self.item("only_tests")["class"], "test_only")
        self.assertEqual(self.item("LONELY")["class"], "zero")
        names = {it["name"] for it in self.census["public"]["detail"]}
        self.assertNotIn("hidden", names, "an item named only in a comment is not an item")
        self.assertEqual(self.census["public"]["narrowable_crate_or_super"], 1)

    def test_use_graph_finds_the_mutual_pair(self):
        self.assertEqual(self.census["graph"]["mutual_pairs"], [["a", "b"]])
        self.assertIn("a->b", self.census["graph"]["edges"])

    def test_shape_counts_tests_and_files(self):
        s = self.census["shape"]
        self.assertEqual(s["files"], 3)
        self.assertEqual(s["test"], 7)
        self.assertEqual(s["lines"], sum(f["lines"] for f in s["per_file"].values()))

    def test_a_test_gated_child_file_is_test_code(self):
        write_tree(self.root, {
            "src/b.rs": TREE["src/b.rs"] + "#[cfg(test)]\nmod more;\n",
            "src/b/more.rs": "use super::*;\n#[test]\nfn t() { let _ = calls(); }\n",
        })
        c = MODULE.census(self.root)
        self.assertEqual(c["shape"]["per_file"]["src/b/more.rs"].get("code", 0), 0)
        self.assertEqual(c["shape"]["per_file"]["src/b/more.rs"]["test"], 4)
        calls = [it for it in c["public"]["detail"] if it["name"] == "calls"][0]
        self.assertEqual(calls["class"], "test_only", "a call from a test child is a test reference")

    def test_duplicates_find_a_copied_run(self):
        body = "".join(f"    let v{i} = {i} * 3 + {i};\n" for i in range(10))
        write_tree(self.root, {"src/c.rs": f"fn one() {{\n{body}}}\n", "src/d.rs": f"fn two() {{\n{body}}}\n"})
        dups = MODULE.duplicates(MODULE.load(self.root))
        self.assertEqual(dups["pairs"], 1)
        self.assertEqual(dups["top"][0]["lines"], 10)

    def test_feature_lines_and_gated_instrumentation(self):
        write_tree(self.root, {"src/e.rs": (
            "pub fn f() {\n"
            "    #[cfg(feature = \"diag\")]\n"
            "    {\n"
            "        count();\n"
            "    }\n"
            "}\n")})
        f = MODULE.features(MODULE.load(self.root))
        self.assertEqual(f["feature_lines"]["diag"], 1)
        self.assertEqual(f["instrumentation"]["diag_elsewhere"], 4)


class ReviewTests(unittest.TestCase):
    def test_compare_reads_only_the_recorded_keys(self):
        got = {"feature_lines": {"tune": 1, "probes": 9}, "n": 2}
        self.assertEqual(MODULE.compare(got, {"feature_lines": {"tune": 1}, "n": 2}), [])
        self.assertEqual(len(MODULE.compare(got, {"n": 3})), 1)

    @unittest.skipUnless(
        subprocess.run(["git", "-C", str(MODULE.REPO), "cat-file", "-e", MODULE.REVIEW_COMMIT + "^{commit}"],
                       capture_output=True).returncode == 0,
        "the review's commit is not in this clone (a shallow checkout)")
    def test_reproduces_the_review_at_its_commit(self):
        with tempfile.TemporaryDirectory() as tmp:
            MODULE.extract(MODULE.REVIEW_COMMIT, Path(tmp))
            head = MODULE.headline(MODULE.census(Path(tmp)))
        self.assertEqual(MODULE.compare(head, MODULE.REVIEW), [])


if __name__ == "__main__":
    unittest.main()
