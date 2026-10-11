"""Fail on a cited commit the repository can no longer reach.

The documents cite commits as backticked hex (`1abeb46`). Under the branch
model of 2026-10-09 a cited commit must stay reachable: from `master`, from a
GitHub pull request (GitHub keeps every PR's commits as refs/pull/<n>/head),
from a tag, or from a local branch while its work is in flight (a gate arm's
hash fails here once its branch is deleted: cite its recipe instead). A hash
that is not a commit here must be a known identifier from somewhere else
(another engine's commit, a tournament ID, a SHA-256 prefix), listed with what
it is in `citation_foreign.tsv`. A hash written with an ellipsis
(`aac92114…`) is a truncated digest (a binary's or file's SHA-256, a git tree)
unless it resolves to a commit here, which then must be reachable like any
other. Everything else is a dead citation.

The PR refs are not fetched by default; fetch them once per clone:

  git fetch origin "+refs/pull/*/head:refs/remotes/origin/pr/*"

  python tools/diag/check_citations.py            # exit 0 clean, 1 on a problem
"""
from __future__ import annotations

import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
FOREIGN = pathlib.Path(__file__).with_name("citation_foreign.tsv")
EXTENSIONS = (".md", ".txt", ".toml", ".ps1", ".py", ".rs", ".json")
# Recipes name their base commit in their file names and diff headers.
SKIP_PREFIXES = ("analysis/arm_patches/",)
CITATION = re.compile(r"`([0-9a-f]{7,40})(…)?`")


def citations(text: str) -> list[tuple[int, str, bool]]:
    """(line number, hash, truncated) for every backticked hex citation;
    `truncated` marks the ellipsis form. Plain numbers are not hashes."""
    found = []
    for number, line in enumerate(text.splitlines(), 1):
        for match in CITATION.finditer(line):
            token = match.group(1)
            if not token.isdigit():
                found.append((number, token, match.group(2) is not None))
    return found


def load_foreign(text: str) -> dict[str, str]:
    """`hash<TAB>what it is` per line; `#` starts a comment."""
    entries: dict[str, str] = {}
    for number, line in enumerate(text.splitlines(), 1):
        if not line.strip() or line.startswith("#"):
            continue
        token, _, what = line.partition("\t")
        if not re.fullmatch(r"[0-9a-f]{7,40}", token) or not what.strip():
            raise ValueError(f"{FOREIGN.name}:{number}: expected `hash<TAB>description`")
        if token in entries:
            raise ValueError(f"{FOREIGN.name}:{number}: {token} listed twice")
        entries[token] = what.strip()
    return entries


def classify(token: str, resolve, reachable: set[str], foreign: dict[str, str],
             truncated: bool = False) -> str | None:
    """None when the citation is fine, otherwise the problem."""
    full = resolve(token)
    if full is not None:
        if full in reachable:
            return None
        return "a commit on no kept ref (not on master, a PR, a tag or a local branch)"
    if token in foreign or truncated:
        return None
    return "not a commit here, and not a listed foreign identifier"


def _git(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True,
                          encoding="utf-8")


def main() -> int:
    refs = _git("for-each-ref", "--format=%(refname)").stdout.split()
    prs = [r for r in refs if r.startswith("refs/remotes/origin/pr/")]
    if not prs:
        print('no PR refs here; fetch them first: git fetch origin "+refs/pull/*/head:refs/remotes/origin/pr/*"')
        return 1
    master = next((r for r in ("refs/remotes/origin/master", "refs/heads/master") if r in refs), None)
    if master is None:
        print("no master ref")
        return 1
    tips = [master, "HEAD", *prs,
            *[r for r in refs if r.startswith(("refs/heads/", "refs/tags/"))]]
    reachable = set(_git("rev-list", *tips).stdout.split())

    def resolve(token: str) -> str | None:
        result = _git("rev-parse", "--verify", "--quiet", f"{token}^{{commit}}")
        return result.stdout.strip() if result.returncode == 0 else None

    foreign = load_foreign(FOREIGN.read_text(encoding="utf-8"))
    files = [f for f in _git("ls-files").stdout.splitlines()
             if f.endswith(EXTENSIONS) and not f.startswith(SKIP_PREFIXES)]
    problems, cited, seen = [], set(), {}
    for name in files:
        try:
            text = (ROOT / name).read_text(encoding="utf-8")
        except (UnicodeDecodeError, FileNotFoundError):
            continue
        for number, token, truncated in citations(text):
            cited.add(token)
            key = (token, truncated)
            if key not in seen:
                seen[key] = classify(token, resolve, reachable, foreign, truncated)
            if seen[key]:
                problems.append(f"{name}:{number}: `{token}`: {seen[key]}")
    for token in sorted(set(foreign) - cited):
        problems.append(f"{FOREIGN.name}: `{token}` is listed but no longer cited")
    for problem in problems:
        print(problem)
    print(f"{len(seen)} cited hashes in {len(files)} files: "
          f"{'clean' if not problems else f'{len(problems)} problem(s)'}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
