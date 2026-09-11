#!/usr/bin/env python3
"""Check GUIDE.md's status board mechanically.

GUIDE is the file that says what to do next, so a wrong checkbox sends the next
session at finished work or hides unfinished work. The guarded failures below
are not reliably visible by reading:

1. **Sub-item indentation.** A sub-item under a `- ` parent must be indented
   **4** spaces. The parent's content column is 2, and an indented code block
   starts 4 columns past that -- so **6 spaces renders as a code block**, not as
   a nested list. That is exactly what happened when the checkboxes were first
   added, and it looks fine in a diff.

2. **A hanging parent.** If every sub-step of a step is ticked, the parent must
   be ticked too. Leaving it open makes finished work look outstanding.

3. **A missing phase.** GUIDE is the maintainer's week-to-week status board and
   must list EVERY phase, not only the one being worked on. Phases 6-9 were
   dropped during a shortening pass on 2026-08-30 and nobody caught it by
   reading; the maintainer did, weeks later.

4. **A SUPERSEDED marker with nobody holding the debt.** A completed step whose
   RESULT was invalidated stays TICKED and carries `SUPERSEDED -> <leaf>`
   naming the open leaf that repairs it. That convention replaced leaving the
   box open, which made the board unrunnable: its first open item was 4.9a.1,
   whose repair simply IS 4.10.1 plus 4.11.1, so nobody could pick it up. The
   marker only works if the owner is real and still open, so all three are
   checked -- the marker may sit only on a ticked leaf, must name a leaf that
   exists, and that leaf must be unticked.

5. **PLAN and GUIDE listing different sub-steps.** The old check was one-way:
   every GUIDE step had to appear in PLAN. So a PLAN item with seven sub-steps
   listed as five in GUIDE passed, and did -- 4.10 was found that way, with
   GUIDE's titles also off by one against PLAN's. Both directions are compared
   now.

6. **Invalid or drifting active workflow metadata.** Open leaves in the active phases (A and B)
   must have one PLAN row using a canonical state/capability class, and GUIDE's
   compact suffix must agree. Vendor/model tags do not belong on those active
   checklist lines; GUIDE's model mapping owns them.

The child pattern is checked against the format GUIDE actually uses --
`- [ ] **A.2.1** ...`, bold, lettered phase, dotted step. The first version
of this checker required a bare `4.9.1`, matched no line in the file, and so
passed vacuously for every edit it existed to guard. The step count in the
success line is there to make that failure mode visible.

There is also an output that is not a failure but is just as invisible: WHICH
LEAF IS NEXT. The board is 146 lines with 42 ticked, so reading the next few
open items off it by eye is exactly the sort of manual step this file exists to
replace. `--next N` prints them, generated from the board rather than copied
into it, so the queue cannot drift from the checkboxes the way a hand-written
list would.

An ACTIONABLE leaf is an unticked item that nobody else discharges: a sub-step,
or a step that has no sub-steps. A parent with children is a heading, not work.

Usage:
  python tools/diag/check_guide.py
  python tools/diag/check_guide.py --next 8
  python tools/diag/check_guide.py --self-test

Exit status is 0 when clean, 1 otherwise, so it can gate a commit. `--next`
does not change it.
"""

import argparse
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
GUIDE = ROOT / "GUIDE.md"

# The step number must be the WHOLE bold run. Writing `**4.9 NEXT**` puts the
# marker inside the bold, the number then fails to match, and the step drops
# out of the count silently -- which is how the count went 100 -> 101 when one
# such marker was moved out. Markers go after the bold: `**4.9** NEXT - ...`.
PARENT = re.compile(r"^- \[([ x])\] \*\*([A-Z]\.\d+)\*\*")
CHILD = re.compile(r"^( *)- \[([ x])\] \*\*([A-Z]\.\d+\.\d+)\*\*")
STRAY = re.compile(r"^ *- \[[ x]\] \*\*[A-Z]\.\d+(\.\d+)? [^*]")
PHASE = re.compile(r"^## Phase ([A-Z])")
REQUIRED_PHASES = set("ABCDEFG")
PLAN = ROOT / "PLAN.md"
# Every GUIDE step number must appear somewhere in PLAN. GUIDE and PLAN are
# required to change in the same commit, and three times in one session a
# scripted PLAN edit matched no anchor, reported success, and was committed with
# a GUIDE that had changed -- leaving the two disagreeing with nothing to catch
# it. This is the cheap half of that check: not that the prose agrees, but that
# PLAN has heard of every step GUIDE lists.
STEP_IN_PLAN = re.compile(r"(?<![\d.])%s(?![\d])")
# `SUPERSEDED -> 4.11.1`, after the closing bold. See failure 4 above.
SUPERSEDED = re.compile(
    r"\*\*([A-Z]\.\d+(?:\.\d+)?)\*\*.*?SUPERSEDED\s*->\s*"
    r"([A-Z]\.\d+(?:\.\d+)?)"
)
# A PLAN sub-step DEFINITION, not a reference to one. PLAN writes a definition
# as `**4.10.1 Some title...**` -- bold, number, space, then the title -- while
# a cross-reference is either bare (`re-derived at 4.11.2`) or bold with
# nothing after the number (`**4.12.22**`). The trailing `\s+\S` is what
# separates them, and without it every owner pointer in the prose would be
# read as a step this file does not define.
PLAN_DEFINITION = re.compile(r"\*\*([A-Z]\.\d+\.\d+)\s+\S")
WORKFLOW_ROW = re.compile(
    r"^\|\s*([A-Z]\.\d+(?:\.\d+)?)\s*\|\s*([A-Z_]+)\s*\|\s*([A-Z]\d?)\s*\|"
)
GUIDE_WORKFLOW = re.compile(
    r"\*\*([A-Z]\.\d+(?:\.\d+)?)\*\*.*?\*\*"
    r"([A-Z_]+)\s*/\s*([A-Z]\d?)\*\*"
)
MODEL_TAG = re.compile(r"\b(?:Astra|Terra|Sol|Opus|Sonnet|Fable)\b")
VALID_STATES = {
    "RESEARCH",
    "READY_FOR_IMPLEMENTATION",
    "IMPLEMENTED",
    "LOCAL_QUALIFIED",
    "GAME_GATE",
    "CLOSED",
}
VALID_CLASSES = {"R3", "R2", "I2", "I1", "M", "V"}
ACTIVE_PREFIXES = ("A.", "B.")


def parse_workflow_rows(lines):
    """Return active PLAN metadata and structural problems."""
    rows = {}
    problems = []
    for n, line in enumerate(lines, 1):
        match = WORKFLOW_ROW.match(line)
        if not match:
            continue
        leaf, state, capability = match.groups()
        if not leaf.startswith(ACTIVE_PREFIXES):
            continue
        if leaf in rows:
            problems.append(
                "PLAN.md:%d: duplicate workflow metadata for %s" % (n, leaf)
            )
        rows[leaf] = (state, capability)
        if state not in VALID_STATES:
            problems.append(
                "PLAN.md:%d: %s has invalid workflow state %s"
                % (n, leaf, state)
            )
        if capability not in VALID_CLASSES:
            problems.append(
                "PLAN.md:%d: %s has invalid capability class %s"
                % (n, leaf, capability)
            )
    return rows, problems


def self_test():
    """Prove the workflow guard rejects intentionally malformed input."""
    sample = [
        "| A.2.1 | WRONG_STATE | R3 | synthetic |",
        "| A.2.1 | RESEARCH | Z9 | duplicate and invalid |",
    ]
    _, problems = parse_workflow_rows(sample)
    expected = ("invalid workflow state", "duplicate workflow", "invalid capability")
    missing = [term for term in expected if not any(term in p for p in problems)]
    if missing:
        sys.stdout.write("FAIL: workflow self-test missed: %s\n" % ", ".join(missing))
        return 1
    sys.stdout.write("workflow metadata negative self-test: PASS (3 failures detected)\n")
    return 0


def actionable(lines):
    """The unticked leaves, in board order, with their trailing text.

    Returns (number, text) pairs. A step with sub-steps is a heading and is
    skipped: ticking it is 4.10.10's hanging-parent rule, not work.
    """
    items = []
    for line in lines:
        m = PARENT.match(line)
        if m:
            items.append((0, m.group(2), m.group(1) == "x",
                          line[m.end():].strip(" -—")))
            continue
        k = CHILD.match(line)
        if k:
            items.append((len(k.group(1)), k.group(3), k.group(2) == "x",
                          line[k.end():].strip(" -—")))
    out = []
    for i, (indent, number, ticked, text) in enumerate(items):
        if indent == 0:
            has_children = i + 1 < len(items) and items[i + 1][0] > 0
            if has_children:
                continue
        if not ticked:
            out.append((number, text))
    return out


def main():
    ap = argparse.ArgumentParser(description="Check GUIDE.md's status board.")
    ap.add_argument("--next", type=int, default=0, metavar="N",
                    help="also print the next N actionable leaves, in board "
                         "order, generated from the checkboxes")
    ap.add_argument("--self-test", action="store_true",
                    help="run an intentionally bad workflow-metadata input")
    args = ap.parse_args()

    if args.self_test:
        return self_test()

    sys.stdout.reconfigure(encoding="utf-8")
    lines = GUIDE.read_text(encoding="utf-8").splitlines()
    problems = []
    parent = None
    kids = []
    phases = set()
    steps = 0
    step_numbers = []

    def close():
        if parent is not None and kids and all(kids) and not parent[1]:
            problems.append(
                "hanging parent: %s has every sub-step ticked but is not ticked"
                % parent[0]
            )

    for n, line in enumerate(lines, 1):
        ph = PHASE.match(line)
        if ph:
            phases.add(ph.group(1))
        m = PARENT.match(line)
        if m:
            close()
            parent = (m.group(2), m.group(1) == "x")
            kids = []
            steps += 1
            step_numbers.append(m.group(2))
            continue
        if STRAY.match(line):
            problems.append(
                "GUIDE.md:%d: text inside the step number's bold run; the "
                "number must be the whole bold (write `**A.4** NEXT - ...`), "
                "or the step drops out of the count silently" % n
            )
        k = CHILD.match(line)
        if k:
            steps += 1
            step_numbers.append(k.group(3))
            indent = len(k.group(1))
            if indent != 4:
                problems.append(
                    "GUIDE.md:%d: sub-item %s indented %d spaces, must be 4 "
                    "(6 renders as an indented code block)"
                    % (n, k.group(3), indent)
                )
            if parent is not None:
                kids.append(k.group(2) == "x")
    close()

    # Failure 4: a SUPERSEDED marker whose owner is missing or already closed.
    ticked = {}
    headings = set()
    last_parent = None
    for line in lines:
        m = PARENT.match(line)
        if m:
            last_parent = m.group(2)
            ticked[last_parent] = m.group(1) == "x"
            continue
        k = CHILD.match(line)
        if k:
            ticked[k.group(3)] = k.group(2) == "x"
            if last_parent is not None:
                headings.add(last_parent)
    for n, line in enumerate(lines, 1):
        marker = SUPERSEDED.search(line)
        if not marker:
            continue
        step, owner = marker.group(1), marker.group(2)
        if not ticked.get(step, False):
            problems.append(
                "GUIDE.md:%d: %s carries SUPERSEDED but is not ticked. The step "
                "was done; it is its RESULT that is superseded" % (n, step)
            )
        if owner not in ticked:
            problems.append(
                "GUIDE.md:%d: %s is SUPERSEDED -> %s, which is not a step on "
                "this board" % (n, step, owner)
            )
        elif ticked[owner]:
            problems.append(
                "GUIDE.md:%d: %s is SUPERSEDED -> %s, but %s is already ticked. "
                "The debt has no owner left" % (n, step, owner, owner)
            )

    plan_text = PLAN.read_text(encoding="utf-8") if PLAN.is_file() else ""
    if not plan_text:
        problems.append("PLAN.md missing; GUIDE and PLAN must change together")
    else:
        absent = [s for s in step_numbers
                  if not re.search(STEP_IN_PLAN.pattern % re.escape(s), plan_text)]
        if absent:
            problems.append(
                "step(s) in GUIDE that PLAN never mentions: %s -- GUIDE and "
                "PLAN change in the same commit" % ", ".join(absent)
            )
        # Failure 5: the other direction. A sub-step PLAN defines and GUIDE
        # does not list is invisible work.
        defined = set(PLAN_DEFINITION.findall(plan_text))
        unlisted = sorted(defined - set(step_numbers))
        if unlisted:
            problems.append(
                "sub-step(s) PLAN defines that GUIDE does not list: %s -- the "
                "board is the file that says what to do next, so work missing "
                "from it does not get done" % ", ".join(unlisted)
            )

        metadata, metadata_problems = parse_workflow_rows(plan_text.splitlines())
        problems.extend(metadata_problems)
        active_open = {
            step for step, is_ticked in ticked.items()
            if not is_ticked and step.startswith(ACTIVE_PREFIXES)
            and step not in headings
        }
        missing_metadata = sorted(active_open - set(metadata))
        if missing_metadata:
            problems.append(
                "open active leaf/leaves missing PLAN workflow metadata: %s"
                % ", ".join(missing_metadata)
            )
        extra_metadata = sorted(set(metadata) - active_open)
        if extra_metadata:
            problems.append(
                "PLAN workflow row(s) are not open active GUIDE leaves: %s"
                % ", ".join(extra_metadata)
            )

        guide_metadata = {}
        for n, line in enumerate(lines, 1):
            item = CHILD.match(line)
            if item:
                if item.group(2) == "x":
                    continue
                leaf = item.group(3)
            else:
                item = PARENT.match(line)
                if not item or item.group(1) == "x" or item.group(2) in headings:
                    continue
                leaf = item.group(2)
            if not leaf.startswith(ACTIVE_PREFIXES):
                continue
            match = GUIDE_WORKFLOW.search(line)
            if not match:
                problems.append(
                    "GUIDE.md:%d: open active leaf %s lacks state/class suffix"
                    % (n, leaf)
                )
                continue
            guide_metadata[leaf] = (match.group(2), match.group(3))
            if MODEL_TAG.search(line):
                problems.append(
                    "GUIDE.md:%d: active leaf %s carries a model tag; use its "
                    "capability class and GUIDE's mapping" % (n, leaf)
                )
        drift = sorted(
            leaf for leaf in active_open
            if leaf in metadata and guide_metadata.get(leaf) != metadata[leaf]
        )
        if drift:
            problems.append(
                "GUIDE/PLAN workflow metadata differs for: %s"
                % ", ".join(drift)
            )

    missing = sorted(REQUIRED_PHASES - phases)
    if missing:
        problems.append(
            "missing phase heading(s): %s -- GUIDE lists every phase, not only "
            "the active one" % ", ".join("Phase %s" % p for p in missing)
        )
    if steps == 0:
        problems.append(
            "no step bullets matched: the GUIDE format changed and this "
            "checker is now passing vacuously"
        )

    if problems:
        for p in problems:
            sys.stdout.write("  %s\n" % p)
        sys.stdout.write("FAIL: %d problem(s) in GUIDE.md\n" % len(problems))
        return 1
    sys.stdout.write(
        "GUIDE.md consistent: %d steps, phases %s\n"
        % (steps, ", ".join(str(p) for p in sorted(phases)))
    )
    if args.next:
        queue = actionable(lines)
        sys.stdout.write(
            "\n%d actionable leaves open. Next %d, in board order:\n"
            % (len(queue), min(args.next, len(queue)))
        )
        for number, text in queue[:args.next]:
            sys.stdout.write("  %-9s %s\n" % (number, text[:78]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
