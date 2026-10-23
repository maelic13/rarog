"""Originality scan: shared numeric runs, scaled numeric tables and shared
token runs between a candidate source tree and donor trees.

usage: python tools/diag/originality_scan.py <out_dir> <cand_tree> <donor_name>=<donor_tree> ...

A tree is a directory holding `src/`, exported with `git archive <rev> src`.
The method and its positive control are in
`analysis/originality_review_2.6.0_2026-10-22.md`.
"""
import os
import re
import sys
import json
from collections import defaultdict

SRC_EXT = {".rs", ".cpp", ".h", ".hpp", ".c"}

COMMENT_RE = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)
STRING_RE = re.compile(r'"(?:\\.|[^"\\])*"')
NUM_RE = re.compile(r"(?<![A-Za-z_0-9.])((?:-[ \t]*)?(?:0x[0-9A-Fa-f_]+|\d[\d_]*(?:\.\d+)?(?:[eE][-+]?\d+)?))(?:[uif](?:8|16|32|64|128|size))?(?![A-Za-z_0-9])")
S_PAIR_RE = re.compile(r"\bS\(\s*(-?\d+)\s*,\s*(-?\d+)\s*\)")
TOKEN_RE = re.compile(r"[A-Za-z_][A-Za-z_0-9]*|0x[0-9A-Fa-f_]+|\d[\d_]*(?:\.\d+)?|::|->|=>|<<=|>>=|<=|>=|==|!=|&&|\|\||<<|>>|\+=|-=|\*=|/=|[{}()\[\];,.:+\-*/%&|^!<>=?#@~]")


def files(tree):
    for root, _, names in os.walk(tree):
        for n in sorted(names):
            if os.path.splitext(n)[1] in SRC_EXT:
                p = os.path.join(root, n)
                yield os.path.relpath(p, tree).replace("\\", "/"), p


def strip(text):
    # Keep line structure: replace comments and strings by same-length blanks.
    def blank(m):
        return re.sub(r"[^\n]", " ", m.group(0))
    text = COMMENT_RE.sub(blank, text)
    return STRING_RE.sub(blank, text)


_LS = {}


def line_of(text, pos):
    import bisect
    ent = _LS.get(id(text))
    if ent is None or ent[0] is not text:
        ent = (text, [m.start() for m in re.finditer("\n", text)])
        _LS[id(text)] = ent
    return bisect.bisect_left(ent[1], pos) + 1


def parse_num(s):
    s = s.replace(" ", "").replace("_", "")
    neg = s.startswith("-")
    body = s[1:] if neg else s
    v = int(body, 16) if body.lower().startswith("0x") else (float(body) if "." in body or "e" in body.lower() else int(body))
    return -v if neg else v


def num_streams(tree):
    """Per file: list of (value, line). Donor C++ files also give S(mg,eg)
    component streams so a split mg/eg table can match."""
    out = {}
    for rel, p in files(tree):
        raw = open(p, encoding="utf-8", errors="replace").read()
        text = strip(raw)
        seq = [(parse_num(m.group(1)), line_of(text, m.start())) for m in NUM_RE.finditer(text)]
        out[rel] = seq
        pairs = [(m, line_of(text, m.start())) for m in S_PAIR_RE.finditer(text)]
        if pairs:
            out[rel + "#mg"] = [(int(m.group(1)), ln) for m, ln in pairs]
            out[rel + "#eg"] = [(int(m.group(2)), ln) for m, ln in pairs]
    return out


def trivial(vals):
    s = set(vals)
    if s <= {0, 1, -1, 2}:
        return True
    diffs = {vals[i + 1] - vals[i] for i in range(len(vals) - 1)}
    if len(diffs) == 1 and abs(next(iter(diffs))) <= 1:
        return True  # 1,2,3,4,... or constant
    if all(isinstance(v, int) and v > 0 and (v & (v - 1)) == 0 for v in vals):
        return True  # powers of two
    return False


def common_runs(a, b, k):
    """Maximal common runs (length >= k) between value lists a and b."""
    av = [x[0] for x in a]
    bv = [x[0] for x in b]
    index = defaultdict(list)
    for j in range(len(bv) - k + 1):
        index[tuple(bv[j:j + k])].append(j)
    runs = []
    seen = set()
    for i in range(len(av) - k + 1):
        for j in index.get(tuple(av[i:i + k]), ()):
            if (i - 1, j - 1) in seen and i > 0 and j > 0 and av[i - 1] == bv[j - 1]:
                seen.add((i, j))
                continue
            seen.add((i, j))
            n = k
            while i + n < len(av) and j + n < len(bv) and av[i + n] == bv[j + n]:
                n += 1
            runs.append((i, j, n))
    return runs


def bracket_arrays(tree):
    """Numeric arrays written as [ ... ] or { ... } with >= 4 integer items."""
    out = []
    arr_re = re.compile(r"[\[{]\s*((?:-?\d+\s*,\s*){3,}-?\d+)\s*,?\s*[\]}]")
    for rel, p in files(tree):
        text = strip(open(p, encoding="utf-8", errors="replace").read())
        for m in arr_re.finditer(text):
            vals = [int(x) for x in re.findall(r"-?\d+", m.group(1))]
            out.append((rel, line_of(text, m.start()), vals))
    return out


def scaled_matches(cand_arrays, donor_streams, min_len=6):
    """Candidate arrays that equal a donor window times a constant k
    (0.25..4, k != 1) to within rounding (|err| <= 1 on every element)."""
    hits = []
    for rel, ln, A in cand_arrays:
        nz = [x for x in A if x != 0]
        if len(A) < min_len or len(set(A)) < 4 or trivial(A):
            continue
        for drel, seq in donor_streams.items():
            B = [x[0] for x in seq]
            if len(B) < len(A) or not all(isinstance(v, int) for v in B):
                continue
            for j in range(len(B) - len(A) + 1):
                W = B[j:j + len(A)]
                if len(set(W)) < 4:
                    continue
                den = sum(w * w for w in W)
                if den == 0:
                    continue
                k = sum(a * w for a, w in zip(A, W)) / den
                if not (0.25 <= k <= 4) or abs(k - 1) < 1e-9:
                    continue
                if all(abs(a - k * w) <= 1.0 for a, w in zip(A, W)):
                    hits.append((rel, ln, A, drel, seq[j][1], W, round(k, 4)))
    return hits


def tokens(tree, normalize):
    out = {}
    for rel, p in files(tree):
        if not rel.endswith(".rs"):
            continue
        text = strip(open(p, encoding="utf-8", errors="replace").read())
        toks = []
        for m in TOKEN_RE.finditer(text):
            t = m.group(0)
            if normalize and (t[0].isalpha() or t[0] == "_"):
                t = "ID"
            toks.append((t, line_of(text, m.start())))
        out[rel] = toks
    return out


def token_runs(cand, donor, k):
    index = defaultdict(list)
    for drel, toks in donor.items():
        tv = [t[0] for t in toks]
        for j in range(len(tv) - k + 1):
            index[hash(tuple(tv[j:j + k]))].append((drel, j))
    res = []
    for crel, toks in cand.items():
        cv = [t[0] for t in toks]
        i = 0
        while i <= len(cv) - k:
            best = None
            for drel, j in index.get(hash(tuple(cv[i:i + k])), ()):
                dv = [t[0] for t in donor[drel]]
                if dv[j:j + k] != cv[i:i + k]:
                    continue
                n = k
                while i + n < len(cv) and j + n < len(dv) and cv[i + n] == dv[j + n]:
                    n += 1
                if best is None or n > best[2]:
                    best = (drel, j, n)
            if best:
                drel, j, n = best
                dtoks = donor[drel]
                res.append({"cand": crel, "cand_lines": [toks[i][1], toks[i + n - 1][1]],
                            "donor": drel, "donor_lines": [dtoks[j][1], dtoks[j + n - 1][1]],
                            "tokens": n})
                i += n
            else:
                i += 1
    return res


def main():
    out_dir, cand_tree = sys.argv[1], sys.argv[2]
    donors = dict(a.split("=", 1) for a in sys.argv[3:])
    os.makedirs(out_dir, exist_ok=True)
    cand_nums = num_streams(cand_tree)
    cand_arrays = bracket_arrays(cand_tree)
    cand_tok_raw = tokens(cand_tree, False)
    cand_tok_norm = tokens(cand_tree, True)
    summary = {"cand_files": len(cand_nums), "cand_numbers": sum(len(v) for v in cand_nums.values()),
               "cand_arrays": len(cand_arrays), "donors": {}}
    for name, tree in donors.items():
        dn = num_streams(tree)
        runs = []
        for crel, cseq in cand_nums.items():
            for drel, dseq in dn.items():
                for i, j, n in common_runs(cseq, dseq, 5):
                    vals = [x[0] for x in cseq[i:i + n]]
                    runs.append({"cand": crel, "cand_line": cseq[i][1], "donor": drel, "donor_line": dseq[j][1],
                                 "len": n, "trivial": trivial(vals), "vals": vals[:40]})
        runs.sort(key=lambda r: (-r["len"], r["cand"]))
        scaled = scaled_matches(cand_arrays, dn)
        rec = {"numeric_runs_nontrivial": [r for r in runs if not r["trivial"]],
               "numeric_runs_trivial_count": sum(r["trivial"] for r in runs),
               "scaled": [dict(zip(["cand", "cand_line", "cand_vals", "donor", "donor_line", "donor_vals", "k"], h)) for h in scaled]}
        if any(f.endswith(".rs") for f, _ in files(tree)):
            dt_raw = tokens(tree, False)
            dt_norm = tokens(tree, True)
            rec["token_runs_raw_k20"] = sorted(token_runs(cand_tok_raw, dt_raw, 20), key=lambda r: -r["tokens"])
            rec["token_runs_norm_k40"] = sorted(token_runs(cand_tok_norm, dt_norm, 40), key=lambda r: -r["tokens"])
            rec["donor_tokens"] = sum(len(v) for v in dt_raw.values())
        rec["donor_numbers"] = sum(len(v) for k, v in dn.items() if "#" not in k)
        json.dump(rec, open(os.path.join(out_dir, name + ".json"), "w"), indent=1)
        summary["donors"][name] = {k: (len(v) if isinstance(v, list) else v) for k, v in rec.items()}
    summary["cand_tokens"] = sum(len(v) for v in cand_tok_raw.values())
    json.dump(summary, open(os.path.join(out_dir, "summary.json"), "w"), indent=1)
    print(json.dumps(summary, indent=1))


if __name__ == "__main__":
    main()
