"""Rarog's live (non-test) eval literals with |v| >= 30 against SF11's eval
files, with SF11's non-eval files and Reckless as chance baselines; the set
sizes are printed so the rates can be compared.

usage: python tools/diag/originality_eval.py <trees_dir> <out.tsv>
where <trees_dir> holds `rarog/`, `sf11/` and `reckless/` source trees."""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from originality_scan import files, strip, NUM_RE, parse_num, line_of  # noqa: E402

T = sys.argv[1]


def lits(tree, pred):
    s = set()
    for rel, p in files(tree):
        if not pred(rel):
            continue
        text = strip(open(p, encoding="utf-8", errors="replace").read())
        s |= {abs(parse_num(m.group(1))) for m in NUM_RE.finditer(text)}
    return {v for v in s if isinstance(v, int) and v >= 30}


EVAL = ("evaluate.cpp", "material.cpp", "pawns.cpp", "psqt.cpp", "endgame.cpp", "types.h", "bitbase.cpp")
sf_eval = lits(os.path.join(T, "sf11"), lambda r: r.split("/")[-1] in EVAL)
sf_rest = lits(os.path.join(T, "sf11"), lambda r: r.split("/")[-1] not in EVAL)
rk = lits(os.path.join(T, "reckless"), lambda r: True)
print("set sizes (|v|>=30): sf11 eval", len(sf_eval), "sf11 non-eval", len(sf_rest), "reckless", len(rk))

rows = []
for rel, p in files(os.path.join(T, "rarog")):
    if not rel.startswith("src/eval/") or rel.endswith("trace.rs"):
        continue
    raw = open(p, encoding="utf-8", errors="replace").read()
    text = strip(raw)
    tm = re.search(r"#\[cfg\(test\)\]\s*mod\s+\w+", text)
    cut = tm.start() if tm else len(text)
    lines = raw.splitlines()
    for m in NUM_RE.finditer(text[:cut]):
        v = parse_num(m.group(1))
        if not isinstance(v, int) or abs(v) < 30 or abs(v) in (64, 100, 128, 256, 1000, 1024):
            continue
        ln = line_of(text, m.start())
        rows.append((rel, ln, v, abs(v) in sf_eval, abs(v) in sf_rest, abs(v) in rk, lines[ln - 1].strip()[:100]))

n = len(rows)
print("rarog live eval literals |v|>=30:", n)
for i, name in ((3, "sf11 eval"), (4, "sf11 non-eval"), (5, "reckless")):
    k = sum(r[i] for r in rows)
    print(f"  in {name}: {k} ({100 * k / n:.1f}%)")
big = [r for r in rows if abs(r[2]) >= 150]
print("rarog live eval literals |v|>=150:", len(big))
for i, name in ((3, "sf11 eval"), (4, "sf11 non-eval"), (5, "reckless")):
    k = sum(r[i] for r in big)
    print(f"  in {name}: {k} ({100 * k / max(len(big), 1):.1f}%)")
with open(sys.argv[2], "w", encoding="utf-8") as f:
    for r in rows:
        f.write("\t".join(str(x) for x in r) + "\n")
