#!/usr/bin/env python3
"""Keep the agent rules within what every tool loads.

An agent tool loads a rule file only up to a size limit and drops the rest
without saying so. Antigravity cuts a rule file at 24,000 bytes, Codex stops
at 32 KiB of project instructions, and a rulebook grows by one rule an
incident. So the rules are a core (`AGENTS.md`) that every tool loads whole
and modules (`docs/rules/*.md`) that an agent reads in full when a trigger in
the core's index fires. This check is what keeps that true:

1. **An auto-loaded set over budget.** The core, and each tool stub with all
   it imports, must fit the core budget. A set is measured as a tool on this
   machine reads it: in UTF-8 bytes with CRLF line endings.
2. **A module over budget.** A module must be readable in one go.
3. **Index and modules out of step.** Every module has an index entry, every
   index entry a module.
4. **An imported module.** A file a stub or the core imports is loaded on
   every task, so an imported module silently spends the core budget.
   Claude Code's `@path` and Antigravity's `@[label](path)` are both read.
5. **A module without its header.** A module opens with its scope and its
   triggers, word for word as the index has them, so the two cannot drift.
6. **A path trigger that names nothing.** A `Paths:` entry must exist.
7. **The orientation read over budget.** An agent orients from PLAN's
   *Active workflow register*, read whole at every orientation; decided
   history that outgrows the budget belongs in `docs/HISTORY.md`.

The budgets are below. Raising one is a maintainer decision, recorded in the
core's last section with its reason.

Usage:
  python tools/diag/check_rules.py
  python tools/diag/check_rules.py --triggered [--base <rev>]
  python tools/diag/check_rules.py --root <another tree>

`--triggered` prints the modules whose `Paths:` trigger the changed files
fire: the files that differ from `--base` (default `master`), the working
tree's changes and its untracked files. It cannot see a `Kind:` or `Before:`
trigger, so it lists the least an agent must have read, not all of it.

Exit status is 0 when clean, 1 otherwise. `--triggered` does not change it.
"""

import argparse
import fnmatch
import os
import pathlib
import re
import subprocess
import sys

CORE = "AGENTS.md"
MODULE_DIR = "docs/rules"
PLAN = "docs/PLAN.md"
# Three quarters of the smallest documented limit, Antigravity's 24,000 bytes
# a rule file (antigravity.google/docs/rules, read 2026-11-25).
CORE_BUDGET = 18_000
MODULE_BUDGET = 18_000
ORIENTATION_BUDGET = 18_000
REGISTER = "### Active workflow register"

# Files a tool loads by itself, beside or instead of the core.
STUB_FILES = ("CLAUDE.md", "CLAUDE.local.md", "GEMINI.md", ".cursorrules",
              ".windsurfrules", ".github/copilot-instructions.md",
              ".junie/guidelines.md", ".agents/AGENTS.md", ".agents/GEMINI.md")
STUB_DIRS = (".claude/rules", ".cursor/rules", ".agents/rules", ".agent/rules",
             ".windsurf/rules", ".github/instructions")
NESTED = ("AGENTS.md", "CLAUDE.md", "GEMINI.md")

FENCE = re.compile(r"(?ms)^(```|~~~).*?^\1[^\n]*$")
SPAN = re.compile(r"`[^`\n]*`")
INCLUDE = re.compile(r"@\[[^\]\n]*\]\(([^)\s]+)\)")
IMPORT = re.compile(r"(?<![\w@/.])@((?:~/|\.{0,2}/)?[\w.-]+(?:/[\w.-]+)*)")
INDEX_HEAD = re.compile(r"^### `(%s/[\w.-]+\.md)`" % re.escape(MODULE_DIR))
HEADER_LINE = re.compile(r"^- (Scope|Kind|Paths|Before): \S")
TOKEN = re.compile(r"`([^`\n]+)`")


def loaded_size(text):
    """Bytes as a tool reads the file from a CRLF working tree."""
    return len(text.replace("\r\n", "\n").replace("\n", "\r\n").encode("utf-8"))


def squash(text):
    return " ".join(text.split())


def imports(path, text, files):
    """The files `text` pulls into the context, as paths in `files`."""
    prose = SPAN.sub(" ", FENCE.sub(" ", text))
    here = os.path.dirname(path)
    found = []
    for target in INCLUDE.findall(prose) + IMPORT.findall(prose):
        for base in (here, ""):
            candidate = os.path.normpath(os.path.join(base, target)).replace("\\", "/")
            if candidate in files and candidate != path and candidate not in found:
                found.append(candidate)
                break
    return found


def closure(start, files):
    """`start` and everything it imports, directly or through an import."""
    seen = [start]
    for path in seen:
        for target in imports(path, files[path], files):
            if target not in seen:
                seen.append(target)
    return seen


def parse_index(core_text):
    """The core's module index: {module path: its Scope/Kind/Paths/Before lines}."""
    index = {}
    current = None
    for line in core_text.splitlines():
        head = INDEX_HEAD.match(line)
        if head:
            current = head.group(1)
            index.setdefault(current, [])
        elif line.startswith("#"):
            current = None
        elif current is not None and HEADER_LINE.match(line):
            index[current].append(squash(line))
    return index


def module_header(text):
    """A module's own Scope/Kind/Paths/Before lines, above its first section."""
    lines = []
    for line in text.splitlines():
        if line.startswith("## "):
            break
        if HEADER_LINE.match(line):
            lines.append(squash(line))
    return lines


def path_triggers(header):
    """The backticked entries of a header's `Paths:` line."""
    for line in header:
        if line.startswith("- Paths:"):
            return TOKEN.findall(line)
    return []


def fires(pattern, changed):
    """The changed files a `Paths:` entry covers."""
    if pattern.endswith("/"):
        return [f for f in changed if f.startswith(pattern)]
    if any(c in pattern for c in "*?["):
        return [f for f in changed if fnmatch.fnmatch(f, pattern)]
    return [f for f in changed if f == pattern]


def auto_loaded_sets(files):
    """{name: [paths]} for every set of files some tool loads unasked."""
    beside = sorted(f for f in files
                    if os.path.dirname(f) in STUB_DIRS
                    or (os.path.basename(f) in NESTED and "/" in f
                        and f not in STUB_FILES))
    sets = {}
    for start in (CORE,) + STUB_FILES:
        if start in files:
            members = closure(start, files)
            for extra in beside:
                members += [m for m in closure(extra, files) if m not in members]
            sets[start] = members
    return sets


def orientation_size(plan_text):
    """Bytes of PLAN's register section, or None when there is not exactly
    one. It ends at the next heading of its level or above."""
    lines = plan_text.replace("\r\n", "\n").split("\n")
    starts = [i for i, line in enumerate(lines) if line.rstrip() == REGISTER]
    if len(starts) != 1:
        return None
    end = len(lines)
    for j in range(starts[0] + 1, len(lines)):
        if re.match(r"#{1,3} ", lines[j]):
            end = j
            break
    return loaded_size("\n".join(lines[starts[0]:end]) + "\n")


def problems(files):
    """`files` maps each path of the tree to its text. Returns the defects."""
    found = []
    if CORE not in files:
        return ["%s is missing" % CORE]
    modules = sorted(f for f in files
                     if os.path.dirname(f) == MODULE_DIR and f.endswith(".md"))
    for name, members in sorted(auto_loaded_sets(files).items()):
        size = sum(loaded_size(files[m]) for m in members)
        if size > CORE_BUDGET:
            found.append("the set %s loads is %s bytes, over the core budget of %s: %s"
                         % (name, format(size, ","), format(CORE_BUDGET, ","),
                            ", ".join(members)))
        for member in members:
            if member in modules:
                found.append("%s imports the module %s; a module is named by its "
                             "path, never imported" % (name, member))
    index = parse_index(files[CORE])
    for module in modules:
        if module not in index:
            found.append("%s is not in %s's index" % (module, CORE))
    for module, lines in sorted(index.items()):
        if module not in files:
            found.append("%s's index names %s, which does not exist" % (CORE, module))
            continue
        size = loaded_size(files[module])
        if size > MODULE_BUDGET:
            found.append("%s is %s bytes, over the module budget of %s"
                         % (module, format(size, ","), format(MODULE_BUDGET, ",")))
        header = module_header(files[module])
        if not any(l.startswith("- Scope:") for l in header) or len(header) < 2:
            found.append("%s lacks its header: a `- Scope:` line and at least one "
                         "of `- Kind:`, `- Paths:`, `- Before:`" % module)
        elif header != lines:
            found.append("%s's header and its index entry in %s differ; the "
                         "triggers are repeated word for word" % (module, CORE))
        for pattern in path_triggers(lines):
            if any(c in pattern for c in "*?["):
                continue
            hit = pattern in files if not pattern.endswith("/") else any(
                f.startswith(pattern) for f in files)
            if not hit:
                found.append("%s: the path trigger `%s` names nothing in the tree"
                             % (module, pattern))
    if "%s bytes" % format(CORE_BUDGET, ",") not in squash(files[CORE]):
        found.append("%s does not state the core budget this check enforces "
                     "(%s bytes)" % (CORE, format(CORE_BUDGET, ",")))
    if PLAN in files:
        size = orientation_size(files[PLAN])
        if size is None:
            found.append("%s: no single '%s' section to measure the orientation "
                         "read" % (PLAN, REGISTER))
        elif size > ORIENTATION_BUDGET:
            found.append("%s's register, the orientation read, is %s bytes, over "
                         "the budget of %s; move decided history out of it to "
                         "docs/HISTORY.md"
                         % (PLAN, format(size, ","), format(ORIENTATION_BUDGET, ",")))
    return found


def triggered(index, changed):
    """{module: the changed files that fire one of its `Paths:` entries}."""
    out = {}
    for module, lines in sorted(index.items()):
        hits = []
        for pattern in path_triggers(lines):
            hits += [f for f in fires(pattern, changed) if f not in hits]
        if hits:
            out[module] = hits
    return out


def git(root, *args):
    out = subprocess.run(("git",) + args, cwd=root, capture_output=True, check=False)
    if out.returncode != 0:
        return None
    return [l for l in out.stdout.decode("utf-8", "replace").splitlines() if l]


def tree_files(root):
    """Every path of the tree: Git's tracked and untracked files, or a walk
    of the directory when `root` is not a repository."""
    listed = git(root, "ls-files", "-co", "--exclude-standard")
    if listed is None:
        listed = []
        for base, dirs, names in os.walk(root):
            dirs[:] = [d for d in dirs if d not in (".git", "target", "node_modules")]
            for name in names:
                listed.append(os.path.relpath(os.path.join(base, name), root)
                              .replace("\\", "/"))
    return sorted(set(listed))


def read_tree(root):
    """Paths of the whole tree, with the text of the files this check reads."""
    files = {}
    for path in tree_files(root):
        rule_file = (path in (CORE, PLAN) or path in STUB_FILES
                     or os.path.dirname(path) in STUB_DIRS + (MODULE_DIR,)
                     or os.path.basename(path) in NESTED)
        full = root / path
        if rule_file and full.is_file():
            files[path] = full.read_bytes().decode("utf-8")
        else:
            files[path] = ""
    return files


def main():
    ap = argparse.ArgumentParser(description="Check the agent rules' budgets and index.")
    ap.add_argument("--root", type=pathlib.Path,
                    default=pathlib.Path(__file__).resolve().parents[2],
                    help="check another tree, such as a scratch copy")
    ap.add_argument("--triggered", action="store_true",
                    help="list the modules the changed paths require")
    ap.add_argument("--base", default="master",
                    help="with --triggered: the revision the work started from")
    args = ap.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")
    files = read_tree(args.root)

    if args.triggered:
        changed = set()
        for listing in (("diff", "--name-only", "%s...HEAD" % args.base),
                        ("diff", "--name-only", "HEAD"),
                        ("ls-files", "-o", "--exclude-standard")):
            got = git(args.root, *listing)
            if got is None:
                sys.stdout.write("FAIL: git %s failed\n" % " ".join(listing))
                return 1
            changed.update(got)
        hits = triggered(parse_index(files.get(CORE, "")), sorted(changed))
        sys.stdout.write("%d changed file(s) since %s\n" % (len(changed), args.base))
        for module, paths in hits.items():
            more = "" if len(paths) <= 3 else " and %d more" % (len(paths) - 3)
            sys.stdout.write("  read %s: %s%s\n" % (module, ", ".join(paths[:3]), more))
        if not hits:
            sys.stdout.write("  no path trigger fired\n")
        sys.stdout.write("Kind and Before triggers are not computed: hold the work "
                         "against the index in %s.\n" % CORE)
        return 0

    found = problems(files)
    if found:
        for p in found:
            sys.stdout.write("  %s\n" % p)
        sys.stdout.write("FAIL: %d problem(s) in the agent rules\n" % len(found))
        return 1
    for name, members in sorted(auto_loaded_sets(files).items()):
        size = sum(loaded_size(files[m]) for m in members)
        sys.stdout.write("  %-28s %6s of %s bytes (%s)\n"
                         % ("loaded by " + name, format(size, ","),
                            format(CORE_BUDGET, ","), " + ".join(members)))
    index = parse_index(files[CORE])
    for module in sorted(index):
        sys.stdout.write("  %-28s %6s of %s bytes\n"
                         % (module, format(loaded_size(files[module]), ","),
                            format(MODULE_BUDGET, ",")))
    if PLAN in files:
        sys.stdout.write("  %-28s %6s of %s bytes\n"
                         % ("PLAN's register",
                            format(orientation_size(files[PLAN]), ","),
                            format(ORIENTATION_BUDGET, ",")))
    sys.stdout.write("agent rules within budget: the core and %d modules\n" % len(index))
    return 0


if __name__ == "__main__":
    sys.exit(main())
