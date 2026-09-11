#!/usr/bin/env python3
"""Build the versioned Phase-4 differential suite (PLAN 4.2).

Deterministic and reproducible: every position is drawn from a source already
in this repository, by a fixed rule, so the suite can be rebuilt byte-identical
and its provenance is auditable. Nothing is hand-typed, which is the point --
a hand-authored FEN that turns out to be illegal or unreachable poisons a
cohort silently.

Output: tools/diag/phase4_suite_v1.epd, one position per line as

    <fen> ; cohort <name> ; src <source>#<index>

The cohort tag is what makes a divergence readable: a counter that separates
only in the endgame cohort is a different finding from one that separates
everywhere.

Rebuild with:  python tools/diag/build_phase4_suite.py
"""

import re
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / "tools" / "diag" / "phase4_suite_v1.epd"

FEN_BODY = r"[1-8pnbrqkPNBRQK/]+ [wb] (?:[KQkq]+|-) (?:[a-h][36]|-) \d+ \d+"


def from_rust(path, limit, cohort):
    """Pull quoted FENs out of a Rust source file, in file order."""
    text = (ROOT / path).read_text(encoding="utf-8")
    seen, out = set(), []
    for m in re.finditer(r'"(%s)"' % FEN_BODY, text):
        fen = m.group(1)
        if fen in seen:
            continue
        seen.add(fen)
        out.append((fen, cohort, "%s#%d" % (path, len(out))))
        if len(out) >= limit:
            break
    return out


def from_epd(path, limit, cohort, stride):
    """Take every `stride`-th line, so the sample spans the book rather than
    its first page. Fixed stride, not RNG: the suite must be reproducible."""
    out = []
    with (ROOT / path).open(encoding="utf-8") as handle:
        for index, line in enumerate(handle):
            if index % stride:
                continue
            fen = line.strip().split(";")[0].strip()
            if not re.fullmatch(FEN_BODY, fen):
                continue
            out.append((fen, cohort, "%s#%d" % (path, index)))
            if len(out) >= limit:
                break
    return out


def from_wac(path, limit, cohort, stride):
    """WAC-style EPD: `<fen> bm <move>; id "WAC.NNN";` with no move counters."""
    out = []
    for index, line in enumerate((ROOT / path).read_text(encoding="utf-8").splitlines()):
        if index % stride:
            continue
        head = line.split(" bm ")[0].split(" am ")[0].strip()
        parts = head.split()
        if len(parts) == 4:
            head = head + " 0 1"
        if not re.fullmatch(FEN_BODY, head):
            continue
        out.append((head, cohort, "%s#%d" % (path, index)))
        if len(out) >= limit:
            break
    return out


def phase_of(fen):
    """Crude material split, only used to separate middlegame from endgame."""
    board = fen.split(" ", 1)[0]
    heavy = sum(board.count(piece) for piece in "qrQR")
    minor = sum(board.count(piece) for piece in "nbNB")
    return "endgame" if heavy + minor <= 4 else "middlegame"


def main():
    entries = []

    # Openings: unbalanced human openings, the same book every gate uses.
    entries += from_epd("tools/books/UHO_Lichess_4852_v1.epd", 12, "opening", 391)

    # Tactics and checks: the WAC suite the `wac` engine command already runs.
    # Its EPD carries no move counters, so they are appended -- WAC positions
    # are analysis diagrams, and 0 1 is the conventional completion.
    entries += from_wac("src/wac.epd", 12, "tactical", 25)

    # Zugzwang: the dedicated regression set. These are the positions where
    # null-move and its verification are supposed to be dangerous, so they are
    # the population NMP counters must be read on.
    entries += from_rust("tests/zugzwang.rs", 8, "zugzwang")

    # Quiet middlegame: WAC positions that are materially deep but whose
    # cohort is decided by material rather than by the tactic they contain.
    deep = from_wac("src/wac.epd", 400, "tmp", 1)
    mids = [e for e in deep if phase_of(e[0]) == "middlegame"]
    entries += [(f, "quiet_middlegame", s) for f, _, s in mids[7::37]][:8]

    # Endgame: the dedicated regression positions. Few, but real -- the UHO
    # book is openings and will never yield an endgame, so drawing one from it
    # by material would have produced an empty cohort that looked populated.
    entries += from_rust("tests/endgames.rs", 8, "endgame")
    entries += [(f, "endgame", s) for f, _, s in deep
                if phase_of(f) == "endgame"][:6]

    seen, lines = set(), []
    for fen, cohort, src in entries:
        if fen in seen:
            continue
        seen.add(fen)
        lines.append("%s ; cohort %s ; src %s" % (fen, cohort, src))

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")

    counts = {}
    for line in lines:
        cohort = line.split("; cohort ")[1].split(" ;")[0]
        counts[cohort] = counts.get(cohort, 0) + 1
    sys.stdout.write("wrote %d positions to %s\n" % (len(lines), OUT))
    for cohort in sorted(counts):
        sys.stdout.write("  %-18s %d\n" % (cohort, counts[cohort]))


if __name__ == "__main__":
    main()
