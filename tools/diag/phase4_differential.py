#!/usr/bin/env python3
"""Run the Phase-4 differential suite against Rarog and the oracle (PLAN 4.2).

Drives both engines over tools/diag/phase4_suite_v1.epd at a fixed depth, one
thread, and joins their `info string diag <name> <value>` output by counter
name and by cohort.

Both engines are driven with STREAMING stdin: the command is written, output is
read until `bestmove`, and only then is the next command sent. Closing stdin
early aborts the search on both engines -- that is why a naive
`printf 'go depth 10\\nquit\\n' | engine` returns a nonsense move even on the
frozen oracle. Do not "simplify" this back into a pipe.

Only counters the spec marks COMPARABLE are differenced. Rarog-only,
oracle-only and not-comparable counters are reported separately and never as a
gap, because a difference there is a definition, not a finding.

Usage:
  python tools/diag/phase4_differential.py \\
      --rarog target/release/rarog.exe \\
      --oracle hybrid/stockfish/src/stockfish.exe \\
      --depth 9
"""

import argparse
import collections
import os
import pathlib
import subprocess
import sys

NL = chr(10)

ROOT = pathlib.Path(__file__).resolve().parents[2]
SUITE = ROOT / "tools" / "diag" / "phase4_suite_v1.epd"

# Counters the spec places in the COMPARABLE core. Anything absent from this
# set is reported but never differenced.
COMPARABLE = set("""
nodes qnodes nodes_in_check
cutoff_quiet cutoff_capture cutoff_first_move
best_rank_1 best_rank_2_3 best_rank_4_7 best_rank_8_plus
move_seen_tt move_seen_good_capture move_seen_quiet move_seen_bad_capture
lmr_applied lmr_research reduction_depth_sum
razor_drop rfp_cut nmp_attempt nmp_cut
nmp_verify_attempt nmp_verify_pass nmp_verify_fail
probcut_nodes probcut_attempt probcut_cut lmp_nodes quiet_futility_prune see_prune
singular_attempt singular_extend_one singular_multicut
main_tt_probes main_tt_hits tt_cut_exact tt_cut_lower tt_cut_upper
tt_bound_not_usable main_store_exact main_store_lower main_store_upper
q_in_check q_tt_hit q_tt_cut q_stand_pat_cut q_move_cut
root_iterations root_best_changes asp_fail_high asp_fail_low
""".split())

# Excluded from the difference with the reason, so a reader is never left
# wondering whether a counter was forgotten.
EXCLUDED = {
    "lmp_prune": "Rarog-only: per MOVE skipped; the oracle can only count per node",
    "check_extensions": "Rarog removed its in-check extension (+30.75); oracle extends",
    "singular_extend_two": "mechanism absent in 9587eeeb",
    "singular_negative_extension": "mechanism absent in 9587eeeb",
    "iid_applied": "oracle-only; Rarog has IIR, a different mechanism",
    "probcut_tt_served": "oracle-only: TT-served ProbCut return, a path Rarog "
                         "does not have. Subtract it before reading the "
                         "oracle's cut rate as a SEARCH conversion",
    "prune_shadow_moves": "SEE families range over different populations (4.7)",
    "prune_shadow_lmp": "see prune_shadow_moves",
    "prune_shadow_futility": "see prune_shadow_moves",
    "prune_shadow_see": "see prune_shadow_moves",
    "prune_shadow_check_exempt": "see prune_shadow_moves",
    "prune_shadow_overlap_two_plus": "see prune_shadow_moves",
}

# Invariants both sides must satisfy. Checked per engine, not across them: if
# one fails, that engine's instrumentation is wrong and the run is void.
INVARIANTS = [
    ("best_rank_1 == cutoff_first_move",
     lambda c: c["best_rank_1"] == c["cutoff_first_move"]),
    ("rank buckets == cutoff_quiet + cutoff_capture",
     lambda c: c["best_rank_1"] + c["best_rank_2_3"] + c["best_rank_4_7"]
     + c["best_rank_8_plus"] == c["cutoff_quiet"] + c["cutoff_capture"]),
    # Was `probcut_cut <= probcut_attempt` until 4.7c prep. That held on both
    # engines and told us nothing, because the two counters were in different
    # units: the oracle's attempt is per MOVE searched, Rarog's was per NODE
    # entered. Cuts are per node on both sides, so the node count is the only
    # denominator this bounds -- and on the oracle the old form can now legally
    # fail, since a TT-served cut has no attempt behind it.
    ("probcut_cut <= probcut_nodes",
     lambda c: c["probcut_cut"] <= c["probcut_nodes"]),
]


def load_suite():
    rows = []
    for line in SUITE.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        fen, _, rest = line.partition(" ; cohort ")
        cohort = rest.split(" ;")[0].strip()
        rows.append((fen.strip(), cohort))
    return rows


def run_engine(exe, positions, depth, env_extra):
    """One process for the whole suite, driven synchronously."""
    env = dict(os.environ)
    env.update(env_extra)
    # Absolute exe, and cwd set to its directory: the oracle loads
    # rarog_hce.dll from beside itself, and a relative exe path would be
    # resolved against that cwd instead of the caller's.
    exe_path = pathlib.Path(exe).resolve()
    proc = subprocess.Popen([str(exe_path)], stdin=subprocess.PIPE,
                            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                            text=True, bufsize=1, env=env,
                            cwd=str(exe_path.parent))
    per_cohort = collections.defaultdict(collections.Counter)
    total = collections.Counter()
    try:
        proc.stdin.write("uci\n")
        proc.stdin.flush()
        while True:
            line = proc.stdout.readline()
            if not line or line.startswith("uciok"):
                break
        for fen, cohort in positions:
            # Clear the TT between positions. Without this the table carries
            # across unrelated positions and the ANSWER changes: on
            # k7/p4p2/P1q1b1p1/3p3p/3Q4/7P/5PP1/1R4K1 Rarog reports mate in 6
            # from depth 9 when searched alone, and cp 1036 when it follows
            # other positions in the same process.
            proc.stdin.write("ucinewgame\nisready\n")
            proc.stdin.flush()
            while True:
                line = proc.stdout.readline()
                if not line or line.startswith("readyok"):
                    break
            proc.stdin.write("position fen %s\ngo depth %d\n" % (fen, depth))
            proc.stdin.flush()
            while True:
                line = proc.stdout.readline()
                if not line:
                    raise RuntimeError("engine closed early on %s" % fen)
                if line.startswith("info string diag "):
                    parts = line.split()
                    name, value = parts[3], int(parts[4])
                    per_cohort[cohort][name] += value
                    total[name] += value
                elif line.startswith("bestmove"):
                    break
        proc.stdin.write("quit\n")
        proc.stdin.flush()
    finally:
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
    return total, per_cohort


def check_invariants(label, counters):
    ok = True
    for name, test in INVARIANTS:
        try:
            passed = test(counters)
        except KeyError:
            continue
        if not passed:
            sys.stdout.write("  INVARIANT FAILED (%s): %s\n" % (label, name))
            ok = False
    return ok


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rarog", required=True)
    ap.add_argument("--oracle", required=True)
    ap.add_argument("--depth", type=int, default=9)
    args = ap.parse_args()

    positions = load_suite()
    sys.stdout.write("suite %s, %d positions, depth %d, 1 thread\n\n"
                     % (SUITE.name, len(positions), args.depth))

    r_total, r_cohort = run_engine(args.rarog, positions, args.depth,
                                   {"RAROG_DIAG_SAMPLE_STRIDE": "1"})
    o_total, o_cohort = run_engine(args.oracle, positions, args.depth, {})

    sys.stdout.write("Invariants (per engine; a failure voids the run)\n")
    ok = check_invariants("rarog", r_total) & check_invariants("oracle", o_total)
    sys.stdout.write("  %s\n\n" % ("all pass" if ok else "SEE FAILURES ABOVE"))

    # At a fixed depth the two engines build different-sized trees, so a raw
    # r/o ratio mostly restates the node ratio. `norm` divides it out: 1.00
    # means "in line with tree size", and only a value far from 1.00 is a
    # real divergence in how often the mechanism fires per node searched.
    scale = (r_total["nodes"] / o_total["nodes"]) if o_total["nodes"] else 1.0
    # 4.6 AUDIT: a qsearch counter must be normalised by QNODES, not by
    # main-search nodes. Rarog runs 1.62x more qsearch per node than the
    # oracle, and that ratio was contaminating every q_* reading: q_tt_cut read
    # 4.25x when it is 2.63x, and q_stand_pat_cut read 1.62x when the two
    # engines are at EXACT parity. Same denominator class as the probcut and
    # lmp corrections, this time inside the runner itself.
    q_scale = (r_total["qnodes"] / o_total["qnodes"]) if o_total["qnodes"] else scale
    sys.stdout.write("node ratio rarog/oracle = %.3f; norm divides it out" % scale + NL)
    sys.stdout.write("QNODE ratio = %.3f; q_* counters are normalised by THAT"
                     % q_scale + NL + NL)
    sys.stdout.write("%-30s %12s %12s %8s %7s  %s"
                     % ("counter", "rarog", "oracle", "r/o", "norm", "flag")
                     + NL)
    sys.stdout.write("-" * 78 + NL)
    rows = []
    for name in sorted(set(r_total) | set(o_total)):
        if name not in COMPARABLE:
            continue
        r, o = r_total[name], o_total[name]
        if not o:
            rows.append((0.0, name, r, o, "-", "-", "oracle zero"))
            continue
        ratio = r / o
        # q_* counters live in the qsearch, so their denominator is qnodes.
        # `qnodes` itself keeps the node denominator: it IS the ratio in
        # question, not a rate within it.
        den = q_scale if (name.startswith("q_") and name != "qnodes") else scale
        norm = ratio / den if den else 0.0
        flag = ""
        if norm >= 2.0 or (norm and norm <= 0.5):
            flag = "**" if (norm >= 3.0 or norm <= 0.34) else "*"
        rows.append((abs((norm or 1.0) - 1.0), name, r, o,
                     "%.3f" % ratio, "%.2f" % norm, flag))
    for _, name, r, o, ratio, norm, flag in sorted(rows, reverse=True):
        sys.stdout.write("%-30s %12d %12d %8s %7s  %s"
                         % (name, r, o, ratio, norm, flag) + NL)
    sys.stdout.write("\nNot differenced (definition differs, not a gap)\n")
    for name in sorted(EXCLUDED):
        if name in r_total or name in o_total:
            sys.stdout.write("  %-30s r=%-9d o=%-9d  %s\n"
                             % (name, r_total[name], o_total[name],
                                EXCLUDED[name]))

    sys.stdout.write("\nPer-cohort first-move cutoff rate\n")
    for cohort in sorted(set(r_cohort) | set(o_cohort)):
        rc, oc = r_cohort[cohort], o_cohort[cohort]
        rd = rc["cutoff_quiet"] + rc["cutoff_capture"]
        od = oc["cutoff_quiet"] + oc["cutoff_capture"]
        sys.stdout.write("  %-18s rarog %6s   oracle %6s\n" % (
            cohort,
            "%.1f%%" % (100.0 * rc["cutoff_first_move"] / rd) if rd else "n/a",
            "%.1f%%" % (100.0 * oc["cutoff_first_move"] / od) if od else "n/a"))

    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
