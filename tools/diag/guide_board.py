#!/usr/bin/env python3
"""Generate GUIDE.md's status board from docs/PLAN.md.

GUIDE and PLAN used to carry the same steps twice, kept in step by hand and by
check_guide.py's comparisons; three times a scripted PLAN edit missed its
anchor while GUIDE changed. The board is now generated, so the two cannot
disagree: PLAN's step heads are the only place a step's title, class and
status are written.

What PLAN must look like (the grammar this reads):

- a phase is a level-2 heading `## Phase X — <name>`; a finished phase's
  heading ends `— CLOSED <date>` and its first paragraph is the one- or
  two-sentence summary the board shows instead of steps;
- a step head is a list item whose bold run reads
  `**<ID> <title> — <class field>.**`, possibly wrapped over several lines.
  The title is what the board shows; the class is the first backticked
  capability class in the field (`R3`, `R2`, `I2`, `I1`, `M`, `V`); a field
  containing DONE, CLOSED or NO_CHANGE (with an optional date) ticks the step;
- a leaf with a row in the active workflow register shows `STATE / CLASS`.

The board sits in GUIDE.md between BEGIN and END below; everything outside
them (checkpoint, holds, prompts) is hand-kept.

Usage:
  python tools/diag/guide_board.py            # rewrite GUIDE's board in place
  python tools/diag/guide_board.py --check    # exit 1 if GUIDE's board is stale
  python tools/diag/guide_board.py --self-test
"""

import argparse
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
GUIDE = ROOT / "GUIDE.md"
PLAN = ROOT / "docs" / "PLAN.md"

BEGIN = ("<!-- board: generated from docs/PLAN.md by "
         "`python tools/diag/guide_board.py`; edit PLAN, never this block -->")
END = "<!-- end of generated board -->"

PHASE = re.compile(r"^## Phase ([A-Z]) — (.+)$")
STEP = re.compile(r"^ *- \*\*([A-Z]\.\d+(?:\.\d+){0,2}) ")
CLASS = re.compile(r"`(R3|R2|I2|I1|M|V)`")
DONE = re.compile(r"\b(DONE|CLOSED|NO_CHANGE)\b(?: (\d{4}-\d{2}-\d{2}))?")
REGISTER_ROW = re.compile(
    r"^\|\s*([A-Z]\.\d+(?:\.\d+){0,2})\s*\|\s*([A-Z_]+)\s*\|\s*([A-Z]\d?)\s*\|")


class PlanError(Exception):
    """PLAN does not follow the grammar the board is generated from."""


def _bold_run(lines, i):
    """The text of the bold run that opens on line i, joined across wraps."""
    joined = lines[i]
    j = i
    while joined.count("**") < 2:
        j += 1
        if j >= len(lines) or not lines[j].strip():
            raise PlanError("docs/PLAN.md:%d: step head's bold run never closes" % (i + 1))
        joined += " " + lines[j].strip()
    start = joined.index("**") + 2
    return joined[start:joined.index("**", start)]


def parse_plan(plan_text):
    """Return the phases in order: (letter, heading, summary or None, steps).

    Each step is a dict with id, title, cls, status (e.g. 'DONE 2026-10-05'
    or None). Raises PlanError on a head the grammar cannot read.
    """
    lines = plan_text.split("\n")
    register = {}
    for line in lines:
        m = REGISTER_ROW.match(line)
        if m and m.group(2) not in ("Workflow state",):
            register[m.group(1)] = (m.group(2), m.group(3))

    phases = []
    current = None
    seen = set()
    for i, line in enumerate(lines):
        ph = PHASE.match(line)
        if ph:
            current = {"letter": ph.group(1), "heading": line, "steps": [],
                       "closed": "— CLOSED" in line, "summary": None}
            phases.append(current)
            if current["closed"]:
                para = []
                for nxt in lines[i + 1:]:
                    if not nxt.strip():
                        if para:
                            break
                        continue
                    if nxt.startswith("#"):
                        break
                    para.append(nxt.rstrip())
                if not para:
                    raise PlanError("%s has no summary paragraph" % line)
                current["summary"] = "\n".join(para)
            continue
        if line.startswith("## "):
            current = None
            continue
        if current is None or current["closed"]:
            continue
        m = STEP.match(line)
        if not m:
            continue
        sid = m.group(1)
        if sid[0] != current["letter"]:
            raise PlanError("docs/PLAN.md:%d: step %s under Phase %s"
                            % (i + 1, sid, current["letter"]))
        if sid in seen:
            raise PlanError("docs/PLAN.md:%d: step %s defined twice" % (i + 1, sid))
        seen.add(sid)
        bold = _bold_run(lines, i)
        rest = bold[len(sid) + 1:]
        if " — " not in rest:
            raise PlanError("docs/PLAN.md:%d: step %s head lacks ' — <class>'"
                            % (i + 1, sid))
        title, field = rest.split(" — ", 1)
        done = DONE.search(field)
        cls = CLASS.search(field)
        if not done and not cls:
            raise PlanError("docs/PLAN.md:%d: step %s head names no capability class"
                            % (i + 1, sid))
        current["steps"].append({
            "id": sid,
            "title": title.strip(),
            "cls": cls.group(1) if cls else None,
            "status": " ".join(g for g in done.groups() if g) if done else None,
            "register": register.get(sid),
        })
    return phases


def build_board(plan_text):
    """The board text that belongs between BEGIN and END ('\\n' newlines)."""
    phases = parse_plan(plan_text)
    items = [s for p in phases for s in p["steps"]]
    now = None
    for k, s in enumerate(items):
        depth = s["id"].count(".")
        has_children = (k + 1 < len(items)
                        and items[k + 1]["id"].startswith(s["id"] + "."))
        if s["status"] is None and not (depth == 1 and has_children):
            now = s
            break
    out = []
    if now:
        out.append("**Now: %s** (`%s`): %s." % (now["id"], now["cls"], now["title"].rstrip(".")))
    else:
        out.append("**Now:** no open step.")
    for p in phases:
        out += ["", p["heading"], ""]
        if p["closed"]:
            out.append(p["summary"])
            continue
        for s in p["steps"]:
            indent = " " * (4 * (s["id"].count(".") - 1))
            if s["status"]:
                box, suffix = "x", s["status"]
            elif s["register"]:
                box, suffix = " ", "**%s / %s**" % s["register"]
            else:
                box, suffix = " ", "**%s**" % s["cls"]
            out.append("%s- [%s] **%s** %s — %s" % (indent, box, s["id"], s["title"], suffix))
    return "\n".join(out) + "\n"


def _split(guide_text):
    if guide_text.count(BEGIN) != 1 or guide_text.count(END) != 1:
        raise PlanError("GUIDE.md must hold exactly one board block between "
                        "the generated-board markers")
    a = guide_text.index(BEGIN) + len(BEGIN)
    b = guide_text.index(END)
    if a > b:
        raise PlanError("GUIDE.md's board markers are out of order")
    return guide_text[:a], guide_text[a:b], guide_text[b:]


def board_problems(guide_text, plan_text):
    """[] when GUIDE's board equals the one PLAN generates, else problems."""
    try:
        _, block, _ = _split(guide_text.replace("\r\n", "\n"))
        want = "\n" + build_board(plan_text.replace("\r\n", "\n"))
    except PlanError as e:
        return [str(e)]
    if block != want:
        return ["GUIDE.md's board is stale against docs/PLAN.md: run "
                "`python tools/diag/guide_board.py`"]
    return []


def write_board():
    raw = GUIDE.read_bytes().decode("utf-8")
    crlf = "\r\n" in raw
    if crlf and raw.count("\r\n") != raw.count("\n"):
        raise PlanError("GUIDE.md has mixed line endings")
    head, _, tail = _split(raw.replace("\r\n", "\n"))
    plan = PLAN.read_text(encoding="utf-8").replace("\r\n", "\n")
    text = head + "\n" + build_board(plan) + tail
    GUIDE.write_bytes((text.replace("\n", "\r\n") if crlf else text).encode("utf-8"))


def self_test():
    plan = "\n".join([
        "## Phase A — Old work — CLOSED 2026-01-01",
        "",
        "Everything was done.",
        "Twice.",
        "",
        "Pointer paragraph, not shown.",
        "",
        "### Active workflow register",
        "",
        "| Leaf | Workflow state | Class | Current decision |",
        "|---|---|---|---|",
        "| B.2 | RESEARCH | R3 | investigating |",
        "",
        "## Phase B — New work",
        "",
        "- **B.1 First step — `I1`, DONE 2026-02-02.** Body.",
        "- **B.2 Second step, wrapped",
        "  over two lines — `R3` investigation.** Body.",
        "- **B.3 Parent — `M`/`V`.** Body.",
        "    - **B.3.1 Child — `V`.** Body.",
        "- Not a step: **B.9** is only cited here.",
        "",
        "## 3. Protocols",
        "- **B.7 Outside any phase — `V`.** Ignored.",
    ])
    want = "\n".join([
        "**Now: B.2** (`R3`): Second step, wrapped over two lines.",
        "",
        "## Phase A — Old work — CLOSED 2026-01-01",
        "",
        "Everything was done.",
        "Twice.",
        "",
        "## Phase B — New work",
        "",
        "- [x] **B.1** First step — DONE 2026-02-02",
        "- [ ] **B.2** Second step, wrapped over two lines — **RESEARCH / R3**",
        "- [ ] **B.3** Parent — **M**",
        "    - [ ] **B.3.1** Child — **V**",
    ]) + "\n"
    got = build_board(plan)
    if got != want:
        sys.stdout.write("FAIL: board self-test\n--- want\n%s--- got\n%s" % (want, got))
        return 1
    guide = "# G\n\n%s\n%s%s\n\nrest\n" % (BEGIN, want, END)
    if board_problems(guide, plan):
        sys.stdout.write("FAIL: board self-test: a current board reads stale\n")
        return 1
    if not board_problems(guide.replace("Child", "Kid"), plan):
        sys.stdout.write("FAIL: board self-test: an edited board reads current\n")
        return 1
    for bad, why in ((plan.replace(" — `V`.** Body.", ".** Body.", 1), "missing ' — '"),
                     (plan.replace("- **B.3 Parent", "- **C.3 Parent"), "wrong phase")):
        try:
            build_board(bad)
        except PlanError:
            continue
        sys.stdout.write("FAIL: board self-test accepted a head with %s\n" % why)
        return 1
    sys.stdout.write("board generator self-test: PASS (output, staleness, 2 malformed heads)\n")
    return 0


def main():
    ap = argparse.ArgumentParser(description="Generate GUIDE.md's board from docs/PLAN.md.")
    ap.add_argument("--check", action="store_true", help="exit 1 if the board is stale")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")
    if args.self_test:
        return self_test()
    try:
        if args.check:
            problems = board_problems(GUIDE.read_text(encoding="utf-8"),
                                      PLAN.read_text(encoding="utf-8"))
            for p in problems:
                sys.stdout.write("  %s\n" % p)
            return 1 if problems else 0
        write_board()
    except PlanError as e:
        sys.stdout.write("FAIL: %s\n" % e)
        return 1
    sys.stdout.write("GUIDE.md board regenerated from docs/PLAN.md\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
