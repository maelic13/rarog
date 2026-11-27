"""Fail when a registration and its result arrive in one commit.

A prediction is frozen by being committed before the build or run it predicts
(`docs/rules/research.md`, *Predictions*). Git shows that only when the
registration is in an earlier commit than the result. This check reads every
commit the branch adds to `master` (merges skipped) and the working tree
against `HEAD`, and fails on:

1. **A ledger entry born resolved.** A commit adds `docs/experiments/<ID>.md`
   carrying a prediction (a `## Prediction` heading or a bold `**Prediction`
   or `**Frozen prediction` lead) and a disposition other than `open`. An
   entry with no prediction, such as an observation recorded after the fact,
   registers nothing and passes.
2. **A registration and a result added together in an analysis.** In one
   `analysis/*.md` a commit adds a registration line (`registered <date>`,
   `Frozen prediction`) and, below it, a result line (`**Calibration`,
   `: the result`). A result above the next read's registration is the normal
   shape and passes. A line counts as added only when the commit's parent
   lacked it and its marker occurs more often than before, so editing a line
   that already carried a registration adds none.

It reads markers, not meaning: a registration written in other words passes
unseen, and separate commits show an order, not that nothing was exposed.
`master` itself is not read, since a squash commit holds a branch's commits.

A commit that fails and has left this machine cannot be split; it is listed
in KNOWN with its reason, and one that has not is split instead.

  python tools/diag/check_freeze.py               # exit 0 clean, 1 on a problem
"""
from __future__ import annotations

import collections
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
ENTRY_DIR = "docs/experiments"
ANALYSIS_DIR = "analysis"

REGISTRATION = re.compile(r"\bregistered \d{4}-\d{2}-\d{2}\b|\bfrozen predictions?\b", re.IGNORECASE)
RESULT = re.compile(r"\*\*Calibration\b|: the result\b")
# A heading or a bold lead, not the words in a sentence: an observation's
# entry may say it has no frozen prediction.
PREDICTION = re.compile(r"^#{2,3} (?:Frozen )?Predictions?\b|\*\*(?:Frozen )?Predictions?\b",
                        re.IGNORECASE | re.MULTILINE)
DISPOSITION = re.compile(r"^- Disposition: ([a-z-]+)", re.MULTILINE)

# Commits that fail and were pushed before this check existed.
KNOWN = {
    "0231c3ed0e1b8de09c42cf56301bcfd790bec8b1":
        "D.9.3's close: its mechanism read's registration and result in one commit, "
        "the case this check was written for",
}


def added(marker: re.Pattern, old: str, new: str) -> list[int]:
    """Line numbers in `new` of the marker lines it adds to `old`: lines `old`
    lacks, whose marker text `new` holds more often than `old` does."""
    def texts(text: str) -> collections.Counter:
        return collections.Counter(m.group(0).lower() for m in marker.finditer(text))

    surplus = texts(new) - texts(old)
    left = collections.Counter(old.splitlines())
    found = []
    for number, line in enumerate(new.splitlines(), 1):
        if left[line] > 0:
            left[line] -= 1
        elif any(surplus[m.group(0).lower()] > 0 for m in marker.finditer(line)):
            found.append(number)
    return found


def entry_problem(old: str | None, new: str) -> str | None:
    """The problem with a ledger entry's change, or None. `old` is None when
    the change adds the entry."""
    if old is not None or not PREDICTION.search(new):
        return None
    disposition = DISPOSITION.search(new)
    if disposition is None or disposition.group(1) == "open":
        return None
    return ("the entry is added with a prediction and already `%s`: register it `open` "
            "in a commit before the read" % disposition.group(1))


def analysis_problem(old: str | None, new: str) -> str | None:
    """The problem with an analysis's change, or None."""
    registered = added(REGISTRATION, old or "", new)
    results = added(RESULT, old or "", new)
    if registered and results and min(registered) < max(results):
        return ("line %d registers and line %d records a result in the same change: "
                "commit the registration before the read" % (min(registered), max(results)))
    return None


def problem(path: str, old: str | None, new: str) -> str | None:
    """The check that applies to `path`, by where it lives."""
    parent, _, name = path.rpartition("/")
    if not name.endswith(".md"):
        return None
    if parent == ENTRY_DIR:
        return entry_problem(old, new)
    if parent == ANALYSIS_DIR:
        return analysis_problem(old, new)
    return None


def _git(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True)


def _text(spec: str) -> str | None:
    """A blob's text (`<rev>:<path>`), or None when it does not exist."""
    result = _git("show", spec)
    return result.stdout.decode("utf-8", "replace") if result.returncode == 0 else None


def _paths(*args: str) -> list[str]:
    result = _git(*args, "--", ENTRY_DIR, ANALYSIS_DIR)
    if result.returncode != 0:
        raise RuntimeError("git %s failed: %s" % (" ".join(args), result.stderr.decode("utf-8", "replace")))
    return [p for p in result.stdout.decode("utf-8", "replace").splitlines() if p]


def commit_problems(rev: str) -> list[str]:
    """One commit's problems, each as `<path>: <problem>`."""
    found = []
    for path in _paths("diff-tree", "--no-commit-id", "--name-only", "-r", "--no-renames",
                       "--diff-filter=AM", rev):
        new = _text("%s:%s" % (rev, path))
        if new is None:
            continue
        what = problem(path, _text("%s^:%s" % (rev, path)), new)
        if what:
            found.append("%s: %s" % (path, what))
    return found


def worktree_problems() -> list[str]:
    """The working tree's problems against HEAD, untracked files included."""
    found = []
    paths = _paths("diff", "--name-only", "--no-renames", "HEAD") + _paths(
        "ls-files", "-o", "--exclude-standard")
    for path in sorted(set(paths)):
        try:
            new = (ROOT / path).read_bytes().decode("utf-8", "replace")
        except (FileNotFoundError, IsADirectoryError):
            continue
        what = problem(path, _text("HEAD:%s" % path), new)
        if what:
            found.append("%s: %s" % (path, what))
    return found


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    refs = _git("for-each-ref", "--format=%(refname)").stdout.decode("utf-8").split()
    master = next((r for r in ("refs/remotes/origin/master", "refs/heads/master") if r in refs), None)
    if master is None:
        print("no master ref")
        return 1
    listed = _git("rev-list", "--no-merges", "--reverse", "%s..HEAD" % master)
    if listed.returncode != 0:
        print("git rev-list failed: %s" % listed.stderr.decode("utf-8", "replace").strip())
        return 1
    commits = listed.stdout.decode("utf-8").split()
    problems, known = [], 0
    for rev in commits:
        found = commit_problems(rev)
        if found and rev in KNOWN:
            known += 1
            continue
        problems += ["%s %s" % (rev[:7], p) for p in found]
    problems += ["working tree %s" % p for p in worktree_problems()]
    for line in problems:
        print(line)
    print("%d commit(s) since %s and the working tree: %s%s"
          % (len(commits), master.rpartition("/")[2],
             "clean" if not problems else "%d problem(s)" % len(problems),
             ", %d known" % known if known else ""))
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
