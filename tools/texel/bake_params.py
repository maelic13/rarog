#!/usr/bin/env python3
"""Bake a complete tuner param dump (`name idx value` lines) into the
`EvalParams` defaults in src/eval/params.rs and src/eval/material.rs.

PST (pst_mg/pst_eg) and material (mg_val/eg_val) are baked via their named
consts (MG_*_PST / EG_*_PST, MG_VAL / EG_VAL) in material.rs; every other field
is an inline array literal in params.rs's `eval_params!` list and is replaced
in place. Comments, structure and each file's line endings are preserved.
Idempotent and verifiable: after baking, a normal build must reproduce the
tune-binary bench for the same dump.

Usage: python tools/texel/bake_params.py <dump.txt>
"""
import re
import sys

# The files this script rewrites. tools/texel/fit_complete.ps1 backs up, diffs
# and restores exactly these; keep the two lists in step.
PARAMS_RS = "src/eval/params.rs"
MATERIAL_RS = "src/eval/material.rs"

PIECES = ["PAWN", "KNIGHT", "BISHOP", "ROOK", "QUEEN", "KING"]
CONST_FIELDS = {"pst_mg", "pst_eg", "mg_val", "eg_val"}


def load_dump(path):
    fields = {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            parts = line.split()
            if len(parts) != 3:
                continue
            name, idx, val = parts[0], int(parts[1]), int(parts[2])
            fields.setdefault(name, {})[idx] = val
    return {n: [d[i] for i in range(len(d))] for n, d in fields.items()}


def fmt_arr(vals):
    return "[" + ", ".join(str(v) for v in vals) + "]"


def replace_const(text, const_name, vals):
    # const NAME: [i32; 64] = [ ... ];   (possibly multi-line)
    pat = re.compile(
        r"(const " + re.escape(const_name) + r":\s*\[i32;\s*\d+\]\s*=\s*)\[.*?\];",
        re.DOTALL,
    )
    # Check that the pattern MATCHED (subn count), not that the text changed —
    # a baked value set identical to the current defaults (e.g. material pinned
    # by an L2-to-prior fit) legitimately produces identical text.
    new, n = pat.subn(lambda m: m.group(1) + fmt_arr(vals) + ";", text, count=1)
    if n != 1:
        raise SystemExit(f"const {const_name} not replaced (n={n})")
    return new


def replace_field(text, field, vals):
    # <indent>field: LEN = <expr>;   (expr possibly multi-line)
    pat = re.compile(
        r"(?m)^(?P<i>[ \t]*)" + re.escape(field) + r":\s*(?P<len>\d+)\s*=\s*.*?;",
        re.DOTALL,
    )
    def sub(m):
        return f"{m.group('i')}{field}: {m.group('len')} = {fmt_arr(vals)};"
    new, n = pat.subn(sub, text, count=1)
    if n != 1:
        raise SystemExit(f"field {field} not replaced (n={n})")
    return new


def read(path):
    # newline="" keeps "\r\n" as is, so the write below preserves the file's
    # line endings.
    with open(path, encoding="utf-8", newline="") as f:
        return f.read()


def write(path, text):
    with open(path, "w", encoding="utf-8", newline="") as f:
        f.write(text)


def main():
    dump = load_dump(sys.argv[1])
    material = read(MATERIAL_RS)
    params = read(PARAMS_RS)

    # PST consts (split the flat 384 into 6 x 64 per piece).
    for phase, field in (("MG", "pst_mg"), ("EG", "pst_eg")):
        flat = dump[field]
        assert len(flat) == 384, field
        for p, piece in enumerate(PIECES):
            material = replace_const(material, f"{phase}_{piece}_PST", flat[p * 64:(p + 1) * 64])
    # Material consts.
    material = replace_const(material, "MG_VAL", dump["mg_val"])
    material = replace_const(material, "EG_VAL", dump["eg_val"])

    # Every other field: inline literal in the weight list.
    for field, vals in dump.items():
        if field in CONST_FIELDS:
            continue
        params = replace_field(params, field, vals)

    # Both files are rewritten only after every replacement matched.
    write(MATERIAL_RS, material)
    write(PARAMS_RS, params)
    print(f"baked {len(dump)} fields")


if __name__ == "__main__":
    main()
