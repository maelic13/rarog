"""Distinctive-constant overlap: every integer/float literal in the candidate
files that is 'distinctive' (not small, not round, not a power of two), and
which donor trees contain the same literal anywhere in their source.

usage: python tools/diag/originality_constants.py <out.tsv> <cand_tree> <glob,...> <name>=<tree> ...
"""
import fnmatch
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from originality_scan import files, strip, NUM_RE, parse_num, line_of  # noqa: E402


def distinctive(v):
    if isinstance(v, float):
        return v not in (0.0, 0.5, 1.0, 2.0, 10.0, 100.0, 1000.0) and abs(v) > 0.001
    a = abs(v)
    if a < 20 or a > 100000:
        return False
    if a & (a - 1) == 0:
        return False
    if a % 100 == 0 or a in (25, 50, 75, 64 - 1, 128 - 1, 255, 1023, 1025):
        return False
    return True


def literal_set(tree):
    s = set()
    for _, p in files(tree):
        text = strip(open(p, encoding="utf-8", errors="replace").read())
        for m in NUM_RE.finditer(text):
            v = parse_num(m.group(1))
            s.add(abs(v))
    return s


def main():
    out, cand, globs = sys.argv[1], sys.argv[2], sys.argv[3].split(",")
    donors = dict(a.split("=", 1) for a in sys.argv[4:])
    sets = {n: literal_set(t) for n, t in donors.items()}
    rows = []
    for rel, p in files(cand):
        if not any(fnmatch.fnmatch(rel, g) for g in globs):
            continue
        raw = open(p, encoding="utf-8", errors="replace").read()
        text = strip(raw)
        lines = raw.splitlines()
        # Skip #[cfg(test)] modules: everything after the first one in the file.
        tm = re.search(r"#\[cfg\(test\)\]\s*mod\s+\w+", text)
        cut = tm.start() if tm else -1
        for m in NUM_RE.finditer(text):
            if cut >= 0 and m.start() > cut:
                break
            v = parse_num(m.group(1))
            if not distinctive(v):
                continue
            ln = line_of(text, m.start())
            hits = [n for n, s in sets.items() if abs(v) in s]
            rows.append((rel, ln, v, hits, lines[ln - 1].strip()[:110]))
    with open(out, "w", encoding="utf-8") as f:
        for r in rows:
            f.write(f"{r[0]}\t{r[1]}\t{r[2]}\t{','.join(r[3])}\t{r[4]}\n")
    n = len(rows)
    print(f"distinctive literals: {n}")
    for name in donors:
        k = sum(name in r[3] for r in rows)
        print(f"  present in {name}: {k} ({100 * k / max(n, 1):.1f}%)")


if __name__ == "__main__":
    main()
