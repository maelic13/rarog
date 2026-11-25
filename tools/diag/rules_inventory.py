#!/usr/bin/env python3
"""Verify the agent-rule inventory against the rulebook it was split from.

On 2026-11-25 the single rulebook (`AGENTS.md`, 29,399 bytes) was split into
a core that every tool loads and modules under `docs/rules/` that are read on
a trigger. `analysis/agent_rules_inventory_2026-11-25.md` records where every
statement of the old rulebook went. This script is the proof that the record
is complete, so that nobody has to establish it by reading:

1. **Coverage.** The old rulebook is cut into statements mechanically: each
   paragraph or top-level bullet is a block, each sentence of a block a
   statement, with the ID `<section>-<block><letter>`. The inventory must
   carry exactly those IDs, each once, each with the statement's own text.
2. **Arrival.** A statement kept as a rule must be found, whitespace aside,
   in the file the inventory names: its old text, or the reworded text the
   inventory records for it. A duplicate must name a statement that arrived.
   Rationale and history must be found in the record the inventory names.

It is a proof of one change, not a standing check: the rules move on and the
inventory does not follow them. Run it on the commit that made the split, or
on any later tree with `--rev <that commit>`:

  python tools/diag/rules_inventory.py --verify
  python tools/diag/rules_inventory.py --verify --rev <commit>
  python tools/diag/rules_inventory.py --dump          # the statements alone
  python tools/diag/rules_inventory.py --self-test

Exit status is 0 when the inventory is complete, 1 otherwise.
"""

import argparse
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
INVENTORY = "analysis/agent_rules_inventory_2026-11-25.md"

SECTIONS = {
    None: "INT",
    "Classify the work": "CLS",
    "Research and implementation": "RES",
    "Judgment toward the CCRL goal": "JDG",
    "Expensive jobs and interruptions": "EXP",
    "Predictions and negative results": "PRD",
    "The one failure mode": "FM",
    "Token-efficient execution": "TOK",
    "Measurement": "MEA",
    "Verification": "VER",
    "Changes": "CHG",
    "Evidence": "EVI",
    "Gating": "GAT",
    "Documents": "DOC",
    "Sequencing and holds": "SEQ",
    "Handing work back": "HND",
}
KINDS = {"invariant", "task", "duplicate", "history"}

# A sentence ends at ., ? or ! (with any closing bracket, quote or emphasis)
# before whitespace and an opening capital, backtick, quote, bracket or
# emphasis. A lower-case continuation ("2.5.0 on", "RAR-S54; on") is not one.
SENTENCE_END = re.compile(r"(?<=[.?!])([)\"*`]*)\s+(?=[A-Z`\"*(\[])")
SUB_ITEM = re.compile(r"^\s+\d+\.\s+")
ROW = re.compile(r"^\| `([A-Z]+-\d+[a-z]+)` \| ([^|]*) \| ([^|]*) \| ([^|]*) \| (.*) \|$")
REWORDED = re.compile(r"^- `([A-Z]+-\d+[a-z]+)` (now|history): (.*)$")


def squash(text):
    """Whitespace collapsed, so a re-wrapped sentence still matches."""
    return " ".join(text.split())


def letters(n):
    """0 -> a, 25 -> z, 26 -> aa."""
    out = ""
    n += 1
    while n:
        n, r = divmod(n - 1, 26)
        out = chr(ord("a") + r) + out
    return out


def sentences(text):
    """Cut one segment of prose into its sentences."""
    out = []
    start = 0
    for m in SENTENCE_END.finditer(text):
        end = m.start() + len(m.group(1))
        out.append(text[start:end].strip())
        start = m.end()
    out.append(text[start:].strip())
    return [s for s in out if s]


def split_units(text):
    """Return the rulebook's statements as (id, first line, last line, text).

    A block is a paragraph or a top-level bullet with its continuation lines
    and numbered sub-items; headings are not statements.
    """
    units = []
    code = SECTIONS[None]
    block_no = 0
    block = []  # (line number, text)

    def flush():
        nonlocal block
        if not block:
            return
        segments = [[]]
        for _, line in block:
            if SUB_ITEM.match(line):
                segments.append([SUB_ITEM.sub("", line)])
            else:
                segments[-1].append(re.sub(r"^- ", "", line).strip())
        n = 0
        for seg in segments:
            for s in sentences(squash(" ".join(seg))):
                units.append(("%s-%d%s" % (code, block_no, letters(n)),
                              block[0][0], block[-1][0], s))
                n += 1
        block = []

    for n, line in enumerate(text.splitlines(), 1):
        if line.startswith("#"):
            flush()
            if line.startswith("## "):
                title = line[3:].strip()
                if title not in SECTIONS:
                    raise SystemExit("unknown section %r at line %d" % (title, n))
                code = SECTIONS[title]
                block_no = 0
            continue
        if not line.strip():
            flush()
            continue
        if line.startswith("- ") or not block:
            flush()
            block_no += 1
        block.append((n, line))
    flush()
    return units


def git_show(rev, path):
    out = subprocess.run(["git", "show", "%s:%s" % (rev, path)], cwd=ROOT,
                         capture_output=True, check=False)
    if out.returncode != 0:
        raise SystemExit("git show %s:%s failed: %s"
                         % (rev, path, out.stderr.decode("utf-8", "replace").strip()))
    return out.stdout.decode("utf-8")


def parse_inventory(text):
    """Return (base revision, rows, rewordings, history notes).

    A row is `| `ID` | lines | kind | destination | text |`. A reworded
    statement has a `- `ID` now: <new text>` line, and a statement whose
    rationale moved a `- `ID` history: <text as recorded>` line.
    """
    base = re.search(r"^Base: `([0-9a-f]{7,40})`", text, re.M)
    rows = {}
    order = []
    now = {}
    history = {}
    problems = []
    for n, line in enumerate(text.splitlines(), 1):
        m = ROW.match(line)
        if m:
            rid, _, kind, dest, body = (g.strip() for g in m.groups())
            if rid in rows:
                problems.append("inventory:%d: %s listed twice" % (n, rid))
            rows[rid] = (kind, dest, body.replace("\\|", "|"))
            order.append(rid)
            continue
        m = REWORDED.match(line)
        if m:
            target = now if m.group(2) == "now" else history
            if m.group(1) in target:
                problems.append("inventory:%d: second '%s' line for %s"
                                % (n, m.group(2), m.group(1)))
            target[m.group(1)] = m.group(3).strip()
    return (base.group(1) if base else None), rows, order, now, history, problems


def verify(units, rows, now, history, read):
    """Return the problems and the totals. `read(path)` gives a destination
    file's text, or None when it does not exist."""
    problems = []
    source = {u[0]: u[3] for u in units}
    for rid in [u[0] for u in units if u[0] not in rows]:
        problems.append("%s is in the rulebook and not in the inventory" % rid)
    for rid in [r for r in rows if r not in source]:
        problems.append("%s is in the inventory and not in the rulebook" % rid)
    cache = {}

    def has(path, text):
        if path not in cache:
            body = read(path)
            cache[path] = None if body is None else squash(
                re.sub(r"(?m)^\s*(?:[-*>]|\d+\.)\s+", " ", body))
        return cache[path] is not None and squash(text) in cache[path]

    arrived = set()
    totals = {}
    for rid, (kind, dest, body) in rows.items():
        if rid not in source:
            continue
        if squash(body) != squash(source[rid]):
            problems.append("%s: the inventory's text is not the rulebook's" % rid)
        if kind not in KINDS:
            problems.append("%s: unknown kind %r" % (rid, kind))
            continue
        if kind == "duplicate":
            continue
        path = dest.strip("`")
        text = now.get(rid, source[rid])
        if kind == "history":
            text = history.get(rid, source[rid])
        if not has(path, text):
            problems.append("%s: not found in %s" % (rid, path))
            continue
        arrived.add(rid)
        key = (kind, path)
        totals[key] = totals.get(key, 0) + 1
        if kind != "history" and rid in history and not has(
                "docs/HISTORY.md", history[rid]):
            problems.append("%s: its history is not in docs/HISTORY.md" % rid)
    for rid, (kind, dest, _) in rows.items():
        if kind != "duplicate":
            continue
        other = dest.strip("`")
        if other not in arrived:
            problems.append("%s: duplicate of %s, which did not arrive" % (rid, other))
        else:
            totals[("duplicate", "")] = totals.get(("duplicate", ""), 0) + 1
    for rid in now:
        if rid not in rows:
            problems.append("a 'now' line names %s, which has no row" % rid)
        elif rows[rid][0] in ("duplicate", "history"):
            problems.append("%s is %s and cannot be reworded" % (rid, rows[rid][0]))
    for rid in history:
        if rid not in rows:
            problems.append("a 'history' line names %s, which has no row" % rid)
    return problems, totals


def self_test():
    """Prove the verifier rejects an inventory that lost a statement."""
    book = ("# T\n\nIntro one. Intro two.\n\n## Changes\n\n"
            "- Never push. The maintainer\n  does (RAR-X1).\n"
            "- Keep tests.\n  1. First case.\n  2. Second case. Its tail.\n")
    units = split_units(book)
    ids = [u[0] for u in units]
    want = ["INT-1a", "INT-1b", "CHG-1a", "CHG-1b", "CHG-2a", "CHG-2b", "CHG-2c", "CHG-2d"]
    if ids != want:
        sys.stdout.write("FAIL: splitter self-test got %r\n" % ids)
        return 1
    rows = {u[0]: ("invariant", "`core`", u[3]) for u in units}
    rows["CHG-2b"] = ("duplicate", "`CHG-2a`", rows["CHG-2b"][2])
    rows["INT-1b"] = ("history", "`hist`", rows["INT-1b"][2])
    files = {"core": "Intro one.\n- Never push. The maintainer does.\n- Keep tests.\n"
                     "  1. First case.\n  2. Second case.\n     Its tail.\n",
             "hist": "Intro two.", "docs/HISTORY.md": "RAR-X1 is why."}
    now = {"CHG-1b": "The maintainer does."}
    hist = {"CHG-1b": "RAR-X1 is why."}
    clean, totals = verify(units, rows, now, hist, files.get)
    if clean or totals.get(("invariant", "core")) != 6:
        sys.stdout.write("FAIL: clean self-test got %r, %r\n" % (clean, totals))
        return 1
    lost = dict(rows)
    del lost["CHG-2d"]
    lost["CHG-9a"] = ("invariant", "`core`", "Invented.")
    moved = dict(files, core=files["core"].replace("Keep tests.", "Keep them."))
    found, _ = verify(units, lost, now, {"CHG-1b": "Not recorded."}, moved.get)
    expected = ("CHG-2d is in the rulebook", "CHG-9a is in the inventory",
                "CHG-2a: not found", "CHG-2b: duplicate of CHG-2a",
                "CHG-1b: its history is not")
    missing = [e for e in expected if not any(e in p for p in found)]
    if missing or len(found) != len(expected):
        sys.stdout.write("FAIL: negative self-test got %r\n" % found)
        return 1
    sys.stdout.write("inventory negative self-test: PASS (%d defects detected)\n" % len(found))
    return 0


def main():
    ap = argparse.ArgumentParser(description="Verify the agent-rule inventory.")
    ap.add_argument("--verify", action="store_true")
    ap.add_argument("--dump", action="store_true",
                    help="print the base rulebook's statements, tab-separated")
    ap.add_argument("--base", help="revision of the rulebook to cut (with --dump)")
    ap.add_argument("--rev", help="read the inventory and the destinations at "
                                  "this revision instead of the working tree")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")

    if args.self_test:
        return self_test()

    def read(path):
        if args.rev:
            out = subprocess.run(["git", "show", "%s:%s" % (args.rev, path)],
                                 cwd=ROOT, capture_output=True, check=False)
            return out.stdout.decode("utf-8") if out.returncode == 0 else None
        p = ROOT / path
        return p.read_text(encoding="utf-8") if p.is_file() else None

    if args.dump:
        for rid, first, last, text in split_units(git_show(args.base or "HEAD", "AGENTS.md")):
            sys.stdout.write("%s\t%d-%d\t%s\n" % (rid, first, last, text))
        return 0
    if not args.verify:
        ap.print_help()
        return 1

    inventory = read(INVENTORY)
    if inventory is None:
        sys.stdout.write("FAIL: %s is missing\n" % INVENTORY)
        return 1
    base, rows, _, now, history, problems = parse_inventory(inventory)
    if base is None:
        sys.stdout.write("FAIL: the inventory names no 'Base: `<commit>`'\n")
        return 1
    units = split_units(git_show(base, "AGENTS.md"))
    found, totals = verify(units, rows, now, history, read)
    problems.extend(found)
    if problems:
        for p in problems:
            sys.stdout.write("  %s\n" % p)
        sys.stdout.write("FAIL: %d problem(s) in the rule inventory\n" % len(problems))
        return 1
    sys.stdout.write("rule inventory complete: %d statements of %s:AGENTS.md, "
                     "%d rows\n" % (len(units), base, len(rows)))
    for (kind, path), n in sorted(totals.items()):
        sys.stdout.write("  %-9s %-28s %d\n" % (kind, path, n))
    sys.stdout.write("  reworded %d, history notes %d\n" % (len(now), len(history)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
