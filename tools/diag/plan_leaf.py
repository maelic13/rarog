#!/usr/bin/env python3
"""Show an agent what to read in docs/PLAN.md, and where.

PLAN is the agents' roadmap and far too long to read whole: the order of
work is one table in it, and a step's text a few dozen lines among
thousands. This prints the two things an orientation needs, with line
ranges, so that each is read with one bounded read and nothing is guessed:

- with no argument, the *Active workflow register* in its row order, which
  is the order of work; its first open row is the current leaf, and a held
  leaf says so in its row;
- with a step's ID, that step: its head, its register row in full, where its
  text and its parent's text are, and its sub-steps.

It reads PLAN by the grammar the board is generated from (`guide_board.py`),
so the current leaf here is the one GUIDE's board calls Now.

Usage:
  python tools/diag/plan_leaf.py              # the register and the current leaf
  python tools/diag/plan_leaf.py D.9.3        # one step
  python tools/diag/plan_leaf.py D.9.3 --text # one step, with its text

Exit status is 0, or 1 when PLAN cannot be read or the step is not in an
open phase.
"""

import argparse
import pathlib
import re
import sys

import guide_board

ROOT = pathlib.Path(__file__).resolve().parents[2]
PLAN = ROOT / "docs" / "PLAN.md"
REGISTER = "### Active workflow register"
# Sections an agent is sent to by name: the roadmap's rules, which the gating
# and research rules cite by number, and the contracts every change keeps.
NAMED = ("## 2. Operating rules", "### Workflow states and capability classes",
         "### Standing contracts", REGISTER)
HEADING = re.compile(r"^(#{1,6}) ")
ROW = re.compile(r"^\|\s*([A-Z]\.\d+(?:\.\d+){0,2})\s*\|\s*([A-Z_]+)\s*\|\s*([A-Z]\d?)\s*\|\s*(.*?)\s*\|\s*$")


def section_span(lines, heading):
    """(first, last) 1-based lines of the section `heading` opens, or None.
    It ends before the next heading of the same or a higher level."""
    for i, line in enumerate(lines):
        if line.rstrip() == heading:
            level = len(HEADING.match(line).group(1))
            for j in range(i + 1, len(lines)):
                m = HEADING.match(lines[j])
                if m and len(m.group(1)) <= level:
                    return i + 1, j
            return i + 1, len(lines)
    return None


def step_spans(lines):
    """{step ID: (first, last)} for every step head of an open phase. A
    step's text runs to the line before the next head that is not one of its
    own sub-steps, or to the next heading."""
    heads = []
    phase_open = False
    for i, line in enumerate(lines):
        if line.startswith("## "):
            phase_open = bool(guide_board.PHASE.match(line)) and "— CLOSED" not in line
            heads.append((i, None))
            continue
        if HEADING.match(line):
            heads.append((i, None))
            continue
        m = guide_board.STEP.match(line)
        if phase_open and m:
            heads.append((i, m.group(1)))
    spans = {}
    for k, (i, sid) in enumerate(heads):
        if sid is None or sid in spans:
            continue
        end = len(lines)
        for j, other in heads[k + 1:]:
            if other is None or not other.startswith(sid + "."):
                end = j
                break
        while end > i + 1 and not lines[end - 1].strip():
            end -= 1
        spans[sid] = (i + 1, end)
    return spans


def register_rows(lines):
    """The register's rows in order: (leaf, state, class, decision, line)."""
    rows = []
    for n, line in enumerate(lines, 1):
        m = ROW.match(line)
        if m and m.group(2) != "Workflow state" and m.group(1) not in [r[0] for r in rows]:
            rows.append(m.groups() + (n,))
    return rows


def size(lines, span):
    """Bytes of a span as a CRLF working tree holds it."""
    return sum(len(l.encode("utf-8")) + 2 for l in lines[span[0] - 1:span[1]])


def where(lines, span):
    return "docs/PLAN.md:%d-%d (%s bytes)" % (span[0], span[1], format(size(lines, span), ","))


def parents(sid):
    """D.9.2.1 -> [D.9.2, D.9]."""
    parts = sid.split(".")
    return [".".join(parts[:k]) for k in range(len(parts) - 1, 1, -1)]


def overview(plan_text):
    """The lines of the no-argument report."""
    lines = plan_text.split("\n")
    steps = {s["id"]: s for p in guide_board.parse_plan(plan_text) for s in p["steps"]}
    spans = step_spans(lines)
    rows = register_rows(lines)
    out = []
    now = next((r for r in rows if r[0] in steps and steps[r[0]]["status"] is None), None)
    if now:
        out.append("Current leaf: %s (%s / %s), %s" % (now[0], now[1], now[2],
                                                      steps[now[0]]["title"]))
        out.append("  its text: %s" % where(lines, spans[now[0]]))
        for parent in parents(now[0]):
            if parent in spans:
                out.append("  under %s: %s" % (parent, where(lines, spans[parent])))
    else:
        out.append("Current leaf: none; the register has no open row")
    span = section_span(lines, REGISTER)
    if span:
        out.append("")
        out.append("Register, in the order of work: %s" % where(lines, span))
        out.append("Read a row in full before acting on it; a held leaf says so there.")
        for leaf, state, cls, decision, n in rows:
            short = decision if len(decision) <= 96 else decision[:95].rstrip() + "…"
            out.append("  %-9s %-25s %-2s %5d  %s" % (leaf, state, cls, n, short))
    out.append("")
    for heading in NAMED[:-1]:
        span = section_span(lines, heading)
        if span:
            out.append("%s: %s" % (heading.lstrip("# "), where(lines, span)))
    return out


def one_step(plan_text, sid, with_text):
    """The lines of the report on one step, or None when it is not there."""
    lines = plan_text.split("\n")
    steps = {s["id"]: s for p in guide_board.parse_plan(plan_text) for s in p["steps"]}
    spans = step_spans(lines)
    if sid not in steps or sid not in spans:
        return None
    step = steps[sid]
    state = step["status"] or ("%s / %s" % step["register"] if step["register"] else step["cls"])
    out = ["%s (%s), %s" % (sid, state, step["title"]),
           "  its text: %s" % where(lines, spans[sid])]
    for parent in parents(sid):
        if parent in spans:
            out.append("  under %s: %s" % (parent, where(lines, spans[parent])))
    kids = [k for k in steps if k.startswith(sid + ".") and k.count(".") == sid.count(".") + 1]
    if kids:
        out.append("  sub-steps: %s" % ", ".join(
            "%s%s" % (k, " (done)" if steps[k]["status"] else "") for k in kids))
    row = [r for r in register_rows(lines) if r[0] == sid]
    if row:
        out.append("  register row, docs/PLAN.md:%d: %s" % (row[0][4], row[0][3]))
    elif step["status"] is None and not kids:
        out.append("  no register row: not an open leaf of the active phase")
    if with_text:
        first, last = spans[sid]
        out.append("")
        out.extend(lines[first - 1:last])
    return out


def main():
    ap = argparse.ArgumentParser(description="Show what to read in docs/PLAN.md.")
    ap.add_argument("step", nargs="?", help="a step's ID, such as D.9.3")
    ap.add_argument("--text", action="store_true", help="print the step's text too")
    args = ap.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")
    if not PLAN.is_file():
        sys.stdout.write("FAIL: docs/PLAN.md is missing\n")
        return 1
    plan_text = PLAN.read_bytes().decode("utf-8").replace("\r\n", "\n")
    try:
        report = one_step(plan_text, args.step, args.text) if args.step else overview(plan_text)
    except guide_board.PlanError as e:
        sys.stdout.write("FAIL: %s\n" % e)
        return 1
    if report is None:
        sys.stdout.write("FAIL: %s is not a step of an open phase in docs/PLAN.md; a "
                         "closed phase's steps are in docs/archive/\n" % args.step)
        return 1
    sys.stdout.write("\n".join(report) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
