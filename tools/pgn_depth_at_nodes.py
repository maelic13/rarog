#!/usr/bin/env python3
"""Per-engine reported depth and time-per-move from a fastchess PGN.

Built for 10.0(b) and kept because it answers a question no internal counter
can: at a FIXED NODE budget, how deep does each engine go, and how fast?

Why fixed nodes makes it decisive. PLAN 10.0's headline observation was "14.6
nominal depth vs Basilisk's 12.7 at identical NPS with equal eval quality" - a
bigger depth number on a thinner tree - but that came from two measurements
taken under different conditions. In a `-Nodes N` match both engines answer the
same positions with the same node budget, so a depth difference is PURELY tree
shape and a time difference is PURELY speed. No modelling, no normalisation.

The 10.0(b) reading (250,000 nodes/move, ~158k moves per engine):

    engine              moves   mean depth   median   s/move   implied nps
    basilisk-1.9.1     158841        13.96     13.0   0.0819     3,051,641
    rarog-2.3.1        158515        16.47     15.0   0.0775     3,223,853

i.e. Rarog reaches 2.5 MORE plies on the same nodes at near-identical speed -
and loses the match by 65 Elo. Depth is not the currency; tree quality is.

⚠ Registered as a progress metric: after 10.4.6's selectivity re-fit, re-run
this. If the re-fit did what 10.0(c) predicts, Rarog's mean depth at 250k nodes
should FALL toward ~14 while its Elo RISES. A re-fit that keeps the depth
advantage has not fixed the over-pruning.

⚠ Reported depth is each engine's own nominal `info depth` and is NOT
comparable across engines in an absolute sense - extension and reduction
conventions differ. It is comparable as a CHANGE within one engine, and as a
qualitative "who reports more plies on the same nodes".

Usage:
    python tools/pgn_depth_at_nodes.py <fastchess.pgn>
"""

import re
import statistics
import sys
from collections import defaultdict

# fastchess writes each move as `{eval/depth time}`; eval may be a mate score.
COMMENT = re.compile(r"\{[+-]?[\dM.]+/(\d+)\s+([\d.]+)s\}")
TAG = re.compile(r'\[(\w+) "(.*)"\]')


def main(path):
    depths = defaultdict(list)
    times = defaultdict(list)
    white = black = None
    first_is_white = True
    body = []
    in_moves = False

    def flush():
        if not (white and black and body):
            return
        # Comments appear in strict move order, so index parity gives the mover
        # once we know which colour moved first in the recorded body.
        for i, m in enumerate(COMMENT.finditer(" ".join(body))):
            mover = white if ((i % 2 == 0) == first_is_white) else black
            depths[mover].append(int(m.group(1)))
            times[mover].append(float(m.group(2)))

    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            s = line.strip()
            tag = TAG.match(s)
            if tag:
                if in_moves:
                    flush()
                    body, in_moves = [], False
                key, val = tag.group(1), tag.group(2)
                if key == "White":
                    white = val
                elif key == "Black":
                    black = val
                elif key == "FEN":
                    # The book position decides who moves first in the body.
                    first_is_white = " w " in val
                continue
            if s:
                in_moves = True
                body.append(s)
    flush()

    if not depths:
        sys.exit(
            "No move comments found. The PGN needs engine annotations - "
            "fastchess writes them by default; a PGN saved without them "
            "carries no depth information."
        )

    nodes = None
    for arg in sys.argv[2:]:
        if arg.startswith("--nodes="):
            nodes = float(arg.split("=", 1)[1])

    header = f"{'engine':24s} {'moves':>8s} {'mean depth':>11s} {'median':>7s} {'s/move':>9s}"
    if nodes:
        header += f" {'implied nps':>13s}"
    print(header)
    for eng in sorted(depths, key=lambda e: -len(depths[e])):
        d, t = depths[eng], times[eng]
        mean_t = statistics.mean(t)
        row = (
            f"{eng:24s} {len(d):8d} {statistics.mean(d):11.2f} "
            f"{statistics.median(d):7.1f} {mean_t:9.4f}"
        )
        if nodes:
            row += f" {nodes / mean_t if mean_t else 0:13,.0f}"
        print(row)


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    main(sys.argv[1])
