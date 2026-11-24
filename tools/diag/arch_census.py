"""The architecture census: the 2026-11-01 engine review's code-volume and
structure counts (`analysis/engine_review_2026-11-01.md` §3.7 and §3.8), as
one tool that re-measures any tree (PLAN D.8.1; E.1 re-measures with it).

What it counts, all on `src/` unless stated:
  - shape: files, lines, and each line's class (in-file test, comment,
    blank, code), from a source with comments and literals masked;
  - the public census: every `pub`, `pub(crate)` and `pub(super)` item
    outside test code, classed by its references from other files: none
    (`zero`), only test code, `tests/`, `benches/`, `xtask`, the texel
    tuner or `main.rs` (`test_only`), or production `src/` (`used`);
  - the `use` graph between top-level modules and its mutual pairs;
  - feature sites: the lines naming `feature = "tune"`, `"diag"`, `"texel"`
    (and `"probes"`, which the review's tree did not have),
    and the non-test lines each feature's instrumentation occupies;
  - functions over 150 lines outside test code;
  - retired roadmap numbers in comments;
  - duplicated runs of eight or more significant lines.

Usage:
  python tools/diag/arch_census.py [--root DIR] [--json OUT] [--detail]
  python tools/diag/arch_census.py --review     # the review's tree, a3dd950
  python tools/diag/arch_census.py --at <commit> # any commit's tree

`--review` extracts `a3dd950` with `git archive` and exits 1 unless every
headline number of the review's tables reproduces.
"""
from __future__ import annotations

import argparse
import collections
import hashlib
import json
import os
import re
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
REVIEW_COMMIT = "a3dd950"
REVIEW = {  # the review's headline numbers at a3dd950
    "src_files": 51,
    "src_lines": 29190,
    "src_test_lines": 6753,
    "src_code_lines": 16950,
    "pub_items": 657,
    "pub_zero": 77,
    "pub_test_only_without_main": 48,
    "narrowable_crate_or_super": 32,
    "feature_lines": {"tune": 55, "diag": 94, "texel": 67},
    "instrumentation": {"diag_rs": 658, "diag_elsewhere": 725, "texel": 514, "tune": 238, "masks_rs": 142},
    "retired_refs": 193,
    "fns_over_150": 11,
    "largest_fn": ["negamax", 1662],
    "mutual_pairs": [["eval", "tt"], ["search", "search_options"]],
}
GROUPS = {"src": "src", "tests": "tests", "benches": "benches", "xtask": "xtask/src", "tuner": "tools/texel-tuner/src"}
FEATURES = ("tune", "diag", "texel", "probes")  # probes: the compact kernel's rate probes, after the review


# --- the masked-source parser ---------------------------------------------

def read(path: Path) -> str:
    return path.read_bytes().decode("utf-8", errors="replace").replace("\r\n", "\n")


def mask(src: str, keep_strings: bool = False) -> str:
    """Comments and string or char literal contents replaced by spaces, newlines
    and length kept, so offsets and line numbers are unchanged."""
    out = list(src)
    n = len(src)
    i = 0

    def blank(a, b):
        for k in range(a, b):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        c = src[i]
        if c == "/" and i + 1 < n and src[i + 1] == "/":
            j = src.find("\n", i)
            j = n if j < 0 else j
            blank(i, j)
            i = j
        elif c == "/" and i + 1 < n and src[i + 1] == "*":
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif src.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            blank(i, j)
            i = j
        elif c == "r" and re.match(r'r#*"', src[i:i + 12]) and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_")):
            m = re.match(r'r(#*)"', src[i:])
            end = '"' + m.group(1)
            j = src.find(end, i + len(m.group(0)))
            j = n if j < 0 else j + len(end)
            if not keep_strings:
                blank(i + len(m.group(0)), j - len(end))
            i = j
        elif c == '"':
            j = i + 1
            while j < n:
                if src[j] == "\\":
                    j += 2
                elif src[j] == '"':
                    break
                else:
                    j += 1
            if not keep_strings:
                blank(i + 1, j)
            i = j + 1
        elif c == "'":
            m = re.match(r"'(\\.[^']*|[^\\'])'", src[i:i + 12])  # a char literal, not a lifetime
            if m:
                if not keep_strings:
                    blank(i + 1, i + len(m.group(0)) - 1)
                i += len(m.group(0))
            else:
                i += 1
        else:
            i += 1
    return "".join(out)


def match_brace(masked: str, open_idx: int) -> int:
    depth = 0
    for k in range(open_idx, len(masked)):
        if masked[k] == "{":
            depth += 1
        elif masked[k] == "}":
            depth -= 1
            if depth == 0:
                return k
    return len(masked) - 1


def line_of(src: str, off: int) -> int:
    return src.count("\n", 0, off) + 1


def cfg_test_regions(masked: str) -> list[tuple[int, int]]:
    """Offsets of each item under `#[cfg(test)]` (or `all(test, ...)`)."""
    regs = []
    for m in re.finditer(r"#\[cfg\((?:test|all\(test,[^\]\n]*\))\)\]", masked):
        k = m.end()
        while k < len(masked) and masked[k] not in "{;":
            k += 1
        if k < len(masked) and masked[k] == "{":
            regs.append((m.start(), match_brace(masked, k)))
        else:
            regs.append((m.start(), k))
    return regs


def in_regions(off: int, regs) -> bool:
    return any(a <= off <= b for a, b in regs)


def classify_lines(src: str, masked: str, regs) -> list[str]:
    """Each line's class: test (inside a cfg(test) item), blank, comment (wholly
    masked) or code."""
    lines, mlines = src.split("\n"), masked.split("\n")
    out, off = [], 0
    for i, line in enumerate(lines):
        o = off
        off += len(line) + 1
        if any(a <= o <= b or (o <= a <= o + len(line)) for a, b in regs):
            out.append("test")
        elif line.strip() == "":
            out.append("blank")
        elif mlines[i].strip() == "":
            out.append("comment")
        else:
            out.append("code")
    return out


def fn_extents(masked: str, src: str):
    """(name, first line, last line, offset) of every fn with a body."""
    res = []
    for m in re.finditer(r"(?<![\w])fn\s+(\w+)", masked):
        i, pd = m.end(), 0
        while i < len(masked):
            ch = masked[i]
            if ch in "([":
                pd += 1
            elif ch in ")]":
                pd -= 1
            elif pd == 0 and ch == ";":
                i = -1
                break
            elif pd == 0 and ch == "{":
                break
            i += 1
        if i == -1 or i >= len(masked):
            continue
        e = match_brace(masked, i)
        res.append((m.group(1), line_of(src, m.start()), line_of(src, e), m.start()))
    return res


def gated_extent(masked: str, start: int) -> tuple[int, int]:
    """The item, statement or argument a cfg attribute ending at `start` gates."""
    i, n = start, len(masked)
    while True:
        while i < n and masked[i].isspace():
            i += 1
        if masked.startswith("#[", i):
            depth = 0
            while i < n:
                if masked[i] == "[":
                    depth += 1
                elif masked[i] == "]":
                    depth -= 1
                    if depth == 0:
                        i += 1
                        break
                i += 1
            continue
        break
    s, pd = i, 0
    while i < n:
        ch = masked[i]
        if ch in "([":
            pd += 1
        elif ch in ")]":
            pd -= 1
            if pd < 0:
                return s, i
        elif pd == 0 and ch in ";,":
            return s, i
        elif pd == 0 and ch == "{":
            e = match_brace(masked, i)
            j = e + 1
            while re.match(r"\s*else\b", masked[j:j + 20]):
                e = match_brace(masked, masked.find("{", j))
                j = e + 1
            return s, e
        elif pd == 0 and ch == "}":
            return s, i
        i += 1
    return s, n


# --- the tree ---------------------------------------------------------------

class Source:
    def __init__(self, root: Path, path: Path, group: str):
        self.path = path.relative_to(root).as_posix()
        self.group = "bin" if self.path == "src/main.rs" else group
        self.src = read(path)
        self.masked = mask(self.src)
        self.regs = cfg_test_regions(self.masked) if group == "src" else []


TEST_CHILD = re.compile(r"#\[cfg\((?:test|all\(test,[^\]\n]*\))\)\]\s*(?:#\[[^\]\n]*\]\s*)*mod (\w+)\s*;")


def test_children(f: Source, root: Path) -> list[str]:
    """The files a test-gated `mod name;` in `f` names: `<dir>/name.rs` for a
    `mod.rs` or `lib.rs`, `<stem>/name.rs` otherwise."""
    p = Path(f.path)
    base = p.parent if p.name in ("mod.rs", "lib.rs", "main.rs") else p.with_suffix("")
    return [(base / f"{m.group(1)}.rs").as_posix() for m in TEST_CHILD.finditer(f.masked)]


def load(root: Path) -> list[Source]:
    out = []
    for group, sub in GROUPS.items():
        base = root / sub
        if not base.is_dir():
            continue
        for d, _, fs in os.walk(base):
            if "target" in Path(d).parts:
                continue
            for f in sorted(fs):
                if f.endswith(".rs"):
                    out.append(Source(root, Path(d) / f, group))
    by_path = {f.path: f for f in out}
    todo = [f for f in out if f.group in ("src", "bin")]
    while todo:  # a test child's own children are test code too
        f = todo.pop()
        for child in test_children(f, root):
            c = by_path.get(child)
            if c is not None and c.regs != [(0, len(c.src))]:
                c.regs = [(0, len(c.src))]
                todo.append(c)
    return sorted(out, key=lambda s: s.path)


def src_of(files):
    return [f for f in files if f.path.startswith("src/")]


def shape(files) -> dict:
    tot, n_lines, per = collections.Counter(), 0, {}
    for f in src_of(files):
        cls = classify_lines(f.src, f.masked, f.regs)
        c = collections.Counter(cls)
        n = len(cls) - (1 if f.src.endswith("\n") else 0)
        per[f.path] = {"lines": n, **c}
        n_lines += n
        tot.update(c)
    return {"files": len(per), "lines": n_lines, "test": tot["test"], "code": tot["code"],
            "comment": tot["comment"], "blank": tot["blank"], "per_file": per}


# --- the public census ----------------------------------------------------

MODS = r'(?:(?:const|unsafe|async|extern\s+"[^"]*")\s+)*'
ITEM = re.compile(r"(?<![\w])pub(\s*\((?:crate|super|self|in [^)]+)\))?\s+" + MODS
                  + r"(fn|struct|enum|const|static|type|trait|mod|union)\s+(\w+)")
IDENT = re.compile(r"[A-Za-z_]\w*")
DECL_BEFORE = re.compile(r"(?:fn|struct|enum|const|static|type|trait|mod|union)\s*$")


def block_kind(header: str) -> str:
    h = re.sub(r"#\[[^\]]*\]", "", header.strip()).strip()
    h = re.sub(r"^pub(\s*\([^)]*\))?\s+", "", h)
    for kw in ("impl", "trait", "mod", "fn", "struct", "enum", "union", "unsafe impl", "extern",
               "unsafe extern", "const fn", "unsafe fn", "pub fn"):
        if h.startswith(kw):
            return "impl" if kw == "unsafe impl" else kw.split()[-1]
    if "fn " in h:
        return "fn"
    if re.match(r"(unsafe\s+)?impl\b", h):
        return "impl"
    return "other"


def blocks_of(masked: str):
    blocks, stack, last = [], [], 0
    for i, ch in enumerate(masked):
        if ch == "{":
            stack.append((i, masked[last:i]))
            last = i + 1
        elif ch == "}":
            if stack:
                o, header = stack.pop()
                blocks.append((o, i, header))
            last = i + 1
        elif ch == ";":
            last = i + 1
    return blocks


def public_census(files) -> dict:
    counts = {}
    for f in files:
        prod, test, qual = collections.Counter(), collections.Counter(), collections.Counter()
        for m in IDENT.finditer(f.masked):
            if DECL_BEFORE.search(f.masked[max(0, m.start() - 12):m.start()]):
                continue
            (test if f.group == "src" and in_regions(m.start(), f.regs) else prod)[m.group(0)] += 1
        for m in re.finditer(r"(?:\.|::)\s*([A-Za-z_]\w*)", f.masked):
            qual[m.group(1)] += 1
        counts[f.path] = (prod, test, qual)
    items = []
    for f in files:
        if f.group not in ("src", "bin"):
            continue
        blocks = blocks_of(f.masked)
        for m in ITEM.finditer(f.masked):
            off = m.start()
            if in_regions(off, f.regs):
                continue
            inner = None
            for o, c, header in blocks:
                if o < off < c and (inner is None or o > inner[0]):
                    inner = (o, c, header)
            ctx = block_kind(inner[2]) if inner else "top"
            ctx = "top" if ctx == "mod" else ctx
            vis = "pub" + (re.sub(r"\s+", "", m.group(1)) if m.group(1) else "")
            items.append({"file": f.path, "line": line_of(f.src, off), "vis": vis, "kind": m.group(2),
                          "name": m.group(3), "ctx": ctx})
    for it in items:
        refs = dict(src_prod=0, src_test=0, tests=0, benches=0, xtask=0, tuner=0, bin=0)
        name, method = it["name"], it["ctx"] in ("impl", "trait")
        for f in files:
            if f.path == it["file"]:
                continue
            prod, test, qual = counts[f.path]
            if method:
                c_prod, c_test = qual.get(name, 0), 0
                if f.group == "src" and f.regs:
                    c_prod = c_test = 0
                    for m in re.finditer(r"(?:\.|::)\s*" + re.escape(name) + r"\b", f.masked):
                        if in_regions(m.start(), f.regs):
                            c_test += 1
                        else:
                            c_prod += 1
            else:
                c_prod, c_test = prod.get(name, 0), test.get(name, 0)
            if f.group == "src":
                refs["src_prod"] += c_prod
                refs["src_test"] += c_test
            else:
                refs[f.group] += c_prod + c_test
        it["refs"] = refs
        it["class"] = "zero" if sum(refs.values()) == 0 else "test_only" if refs["src_prod"] == 0 else "used"
    by = collections.Counter(it["class"] for it in items)
    return {"items": len(items), "zero": by["zero"], "test_only": by["test_only"], "used": by["used"],
            "test_only_without_main": sum(1 for it in items if it["class"] == "test_only" and it["refs"]["bin"] == 0),
            "narrowable_crate_or_super": sum(1 for it in items if it["class"] == "zero" and it["vis"] != "pub"),
            "detail": items}


# --- the use graph ----------------------------------------------------------

def modpath(rel: str) -> tuple:
    parts = rel[4:-3].split("/")
    return tuple(parts[:-1] if parts[-1] == "mod" else parts)


def expand_use(tree: str) -> list[list[str]]:
    tree = tree.strip()
    idx = tree.find("{")
    if idx < 0:
        return [[s.strip() for s in tree.split("::") if s.strip()]] if tree else []
    prefix = [s.strip() for s in tree[:idx].split("::") if s.strip()]
    d = 0
    for j in range(idx, len(tree)):
        d += {"{": 1, "}": -1}.get(tree[j], 0)
        if d == 0:
            break
    parts, cur, d = [], "", 0
    for ch in tree[idx + 1:j]:
        d += {"{": 1, "}": -1}.get(ch, 0)
        if ch == "," and d == 0:
            parts.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        parts.append(cur)
    return [prefix + e for p in parts for e in expand_use(p)]


def use_graph(files) -> dict:
    mods = {}
    for f in src_of(files):
        mods[() if f.path == "src/lib.rs" else ("__main__",) if f.path == "src/main.rs" else modpath(f.path)] = f.path

    def resolve(segs, cur, is_main):
        segs = [re.sub(r"\s+as\s+\w+$", "", s) for s in segs]
        if not segs:
            return None
        if segs[0] == "crate" or (is_main and segs[0] == "rarog"):
            path = segs[1:]
        elif segs[0] == "self":
            path = list(cur) + segs[1:]
        elif segs[0] == "super":
            base, rest = list(cur), segs
            while rest and rest[0] == "super":
                base, rest = base[:-1], rest[1:]
            path = base + rest
        else:
            return None
        for k in range(len(path), -1, -1):
            if tuple(path[:k]) in mods:
                return mods[tuple(path[:k])]
        return None

    def top(rel):
        p = rel[4:]
        return "lib" if p == "lib.rs" else "main" if p == "main.rs" else p.split("/")[0].replace(".rs", "")

    edges = collections.Counter()
    use_re = re.compile(r"(?<![\w:])use\s+([^;]+);", re.S)
    path_re = re.compile(r"(?<![\w:])(crate|rarog)((?:::\w+)+)")
    for f in src_of(files):
        is_main = f.path == "src/main.rs"
        cur = ("__main__",) if is_main else (() if f.path == "src/lib.rs" else modpath(f.path))
        spans = []
        for um in use_re.finditer(f.masked):
            spans.append((um.start(), um.end()))
            if in_regions(um.start(), f.regs):
                continue
            for segs in expand_use(um.group(1)):
                dst = resolve(segs, cur, is_main)
                if dst and dst != f.path:
                    edges[(top(f.path), top(dst))] += 1
        for pm in path_re.finditer(f.masked):
            if any(a <= pm.start() < b for a, b in spans) or in_regions(pm.start(), f.regs):
                continue
            dst = resolve([pm.group(1)] + [s for s in pm.group(2).split("::") if s], cur, is_main)
            if dst and dst != f.path:
                edges[(top(f.path), top(dst))] += 1
    adj = collections.defaultdict(set)
    for (a, b), n in edges.items():
        if a != b and n:
            adj[a].add(b)

    def reaches(a, b):
        seen, todo = set(), [a]
        while todo:
            x = todo.pop()
            if x == b:
                return True
            for y in adj.get(x, ()):
                if y not in seen:
                    seen.add(y)
                    todo.append(y)
        return False

    nodes = sorted(set(adj) | {y for v in adj.values() for y in v})
    pairs = [[a, b] for i, a in enumerate(nodes) for b in nodes[i + 1:] if reaches(a, b) and reaches(b, a)]
    return {"edges": {f"{a}->{b}": n for (a, b), n in sorted(edges.items()) if a != b}, "mutual_pairs": pairs}


# --- features, functions, comments, duplicates ------------------------------

def features(files) -> dict:
    lines = {k: 0 for k in FEATURES}
    inst = collections.Counter()
    for f in src_of(files):
        for line in f.src.split("\n"):
            for k in FEATURES:
                if f'feature = "{k}"' in line:
                    lines[k] += 1
        cls = classify_lines(f.src, f.masked, f.regs)
        gated = {k: set() for k in FEATURES}
        for am in re.finditer(r"#\[cfg\(([^\n]*?)\)\]", f.src):
            if in_regions(am.start(), f.regs):
                continue
            for k in FEATURES:
                if re.search(r'feature\s*=\s*"%s"' % k, am.group(1)):
                    _, e = gated_extent(f.masked, am.end())
                    gated[k].update(range(line_of(f.src, am.start()), line_of(f.src, e) + 1))
        mlines = f.masked.split("\n")
        bare = {i + 1 for i in range(len(mlines))
                if re.search(r"\b(diag_count|diag_add)!", mlines[i]) and (i + 1) not in gated["diag"]}
        for mm in re.finditer(r"trace_decision!\(", f.masked):
            depth, k = 0, mm.end() - 1
            while k < len(f.masked):
                depth += {"(": 1, ")": -1}.get(f.masked[k], 0)
                if f.masked[k] == ")" and depth == 0:
                    break
                k += 1
            bare.update(ln for ln in range(line_of(f.src, mm.start()), line_of(f.src, k) + 1) if ln not in gated["diag"])
        hooks = {line_of(f.src, mm.start()) for mm in re.finditer(r"\btr_(mg|eg)!\(", f.masked)}

        def nontest(lns):
            return len([ln for ln in lns if cls[ln - 1] != "test"])

        nontest_lines = len([c for c in cls[:len(cls) - (1 if f.src.endswith("\n") else 0)] if c != "test"])
        if f.path == "src/diag.rs":
            inst["diag_rs"] += nontest_lines
        else:
            inst["diag_elsewhere"] += nontest(gated["diag"] | bare)
        if f.path == "src/eval/masks.rs":  # the whole file, as the review counted it
            inst["masks_rs"] += len(cls) - (1 if f.src.endswith("\n") else 0)
        inst["tune"] += nontest(gated["tune"])
        inst["texel"] += nontest(gated["texel"]) + len(hooks)
    return {"feature_lines": lines, "instrumentation": dict(inst)}


def functions(files) -> dict:
    fns = []
    for f in src_of(files):
        for name, a, b, off in fn_extents(f.masked, f.src):
            if not in_regions(off, f.regs):
                fns.append((b - a + 1, f.path, a, name))
    fns.sort(reverse=True)
    return {"count": len(fns), "over_150": sum(1 for x in fns if x[0] > 150),
            "largest": [{"lines": n, "file": p, "line": a, "name": name} for n, p, a, name in fns[:12]]}


RETIRED = {"phase": r"\bPhase \d+", "ledger": r"\bRAR-[A-Z]\d+",
           "old_step": r"\b(?:3|4|5|6|7|8|9|10)\.\d+[a-z]?\b", "step": r"\b[A-G]\.\d+(?:\.\d+)*\b"}


def retired_refs(files) -> dict:
    tot = collections.Counter()
    for f in src_of(files):
        com = []
        for ls, lm in zip(f.src.split("\n"), f.masked.split("\n")):
            idx = ls.find("//")
            while 0 <= idx < len(lm) and lm[idx:idx + 2] == "//":
                idx = ls.find("//", idx + 2)
            if idx >= 0:
                com.append(ls[idx:])
        text = "\n".join(com)
        for k, p in RETIRED.items():
            tot[k] += len(re.findall(p, text))
    return {"total": sum(tot.values()), **tot}


def duplicates(files, window: int = 8) -> dict:
    trivial = re.compile(r"^[\s\{\}\(\)\[\];,]*$")
    norm = {}
    for f in src_of(files):
        m = mask(f.src, keep_strings=True)
        out, off = [], 0
        for i, line in enumerate(m.split("\n")):
            st, off = off, off + len(line) + 1
            if in_regions(st, f.regs):
                continue
            s = re.sub(r"\s+", " ", line).strip()
            if s and not trivial.match(s):
                out.append((i + 1, s))
        norm[f.path] = out
    index = collections.defaultdict(list)
    for p, ls in norm.items():
        for i in range(len(ls) - window + 1):
            index[hashlib.md5("\n".join(s for _, s in ls[i:i + window]).encode()).hexdigest()].append((p, i))
    diag = collections.defaultdict(list)
    for occ in index.values():
        for a in range(len(occ)):
            for b in range(a + 1, len(occ)):
                (fa, ia), (fb, ib) = occ[a], occ[b]
                if not (fa == fb and abs(ia - ib) < window):
                    diag[(fa, fb, ia - ib)].append(ia)
    runs = []
    for (fa, fb, d), starts in diag.items():
        starts = sorted(set(starts))
        begin = prev = starts[0]
        for s in starts[1:] + [None]:
            if s is None or s != prev + 1:
                length = prev - begin + window
                runs.append((length, fa, norm[fa][begin][0], fb, norm[fb][begin - d][0]))
                begin = s
            prev = s if s is not None else prev
    runs.sort(reverse=True)
    seen, pairs = set(), []
    for length, fa, a0, fb, b0 in runs:
        key = tuple(sorted([(fa, a0), (fb, b0)]))
        if key not in seen:
            seen.add(key)
            pairs.append({"lines": length, "a": f"{fa}:{a0}", "b": f"{fb}:{b0}"})
    return {"window": window, "pairs": len(pairs), "lines": sum(p["lines"] for p in pairs), "top": pairs[:20]}


# --- the report -------------------------------------------------------------

def census(root: Path) -> dict:
    files = load(root)
    return {"shape": shape(files), "public": public_census(files), "graph": use_graph(files),
            "features": features(files), "functions": functions(files), "retired": retired_refs(files),
            "duplicates": duplicates(files)}


def headline(c: dict) -> dict:
    top = c["functions"]["largest"][0]
    return {
        "src_files": c["shape"]["files"], "src_lines": c["shape"]["lines"],
        "src_test_lines": c["shape"]["test"], "src_code_lines": c["shape"]["code"],
        "pub_items": c["public"]["items"], "pub_zero": c["public"]["zero"],
        "pub_test_only_without_main": c["public"]["test_only_without_main"],
        "narrowable_crate_or_super": c["public"]["narrowable_crate_or_super"],
        "feature_lines": c["features"]["feature_lines"],
        "instrumentation": {k: c["features"]["instrumentation"].get(k, 0) for k in REVIEW["instrumentation"]},
        "retired_refs": c["retired"]["total"], "fns_over_150": c["functions"]["over_150"],
        "largest_fn": [top["name"], top["lines"]], "mutual_pairs": c["graph"]["mutual_pairs"],
    }


def extract(commit: str, into: Path) -> None:
    tar = into / "tree.tar"
    paths = [p for p in GROUPS.values()]
    listed = subprocess.run(["git", "-C", str(REPO), "ls-tree", "--name-only", commit, "--", *paths],
                            capture_output=True, text=True, check=True).stdout.split()
    subprocess.run(["git", "-C", str(REPO), "archive", "--format=tar", "-o", str(tar), commit, "--", *listed], check=True)
    with tarfile.open(tar) as t:
        t.extractall(into, filter="data")


def compare(got: dict, want: dict) -> list[str]:
    """The keys `want` records; a nested record is compared on its own keys."""
    misses = []
    for k, v in want.items():
        g = got.get(k)
        if isinstance(v, dict) and isinstance(g, dict):
            g = {key: g.get(key) for key in v}
        if g != v:
            misses.append(f"{k}: {g!r}, the review {v!r}")
    return misses


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--root", type=Path, default=REPO)
    ap.add_argument("--json", type=Path, help="write the full census here")
    ap.add_argument("--detail", action="store_true", help="print the largest functions, the graph and the duplicates")
    ap.add_argument("--review", action="store_true", help=f"reproduce the review's numbers at {REVIEW_COMMIT}")
    ap.add_argument("--at", metavar="COMMIT", help="measure this commit's tree instead of --root")
    args = ap.parse_args(argv)
    if args.review or args.at:
        with tempfile.TemporaryDirectory() as tmp:
            extract(REVIEW_COMMIT if args.review else args.at, Path(tmp))
            c = census(Path(tmp))
    else:
        c = census(args.root)
    h = headline(c)
    print(json.dumps(h, indent=1))
    if args.detail:
        for f in c["functions"]["largest"]:
            print(f"  {f['lines']:5d}  {f['file']}:{f['line']}  {f['name']}")
        for e, n in c["graph"]["edges"].items():
            print(f"  {e:34s} {n}")
        print(f"  duplicates: {c['duplicates']['pairs']} pairs, {c['duplicates']['lines']} lines")
        for p in c["duplicates"]["top"]:
            print(f"    {p['lines']:4d}  {p['a']}  ==  {p['b']}")
    if args.json:
        args.json.write_text(json.dumps(c, indent=1), encoding="utf-8")
    if args.review:
        misses = compare(h, REVIEW)
        for line in misses:
            print("MISMATCH", line)
        print("review reproduced" if not misses else f"{len(misses)} headline numbers differ")
        return 1 if misses else 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
