#!/usr/bin/env python3
"""Tests for pricing_table.py: the parser sums every search's summary line,
refuses a malformed record, and the planted comparison separates only when
the intervals are disjoint."""

from __future__ import annotations

import math
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

from pricing_table import main, mantel_haenszel, parse, table, wilson  # noqa: E402


def record(rule="lmr", depth=4, label=0, changed="-", nodes=10, move=5, quiet="1"):
    return f"{rule}\t{depth}\t7\tcut\t{move}\t0\t{nodes}\t{label}\t{changed}\t{quiet}"


def test_summary_lines_are_summed_over_every_search():
    lines = ["# search nodes 100 probes 1 probe_nodes 10", record(),
             "# search nodes 250 probes 2 probe_nodes 30", record(label=1, changed="1"),
             record()]
    read = parse(lines)
    assert (read.main_nodes, read.probe_nodes, read.searches) == (350, 40, 2)
    assert len(read.records) == 3


def test_a_record_with_a_missing_field_is_refused():
    with pytest.raises(ValueError, match="fields"):
        parse(["lmr\t4\t7\tcut\t5\t0\t10\t1\t-"])


def test_a_bad_label_is_refused():
    with pytest.raises(ValueError, match="label"):
        parse([record().replace("\t0\t-\t", "\t2\t-\t")])


def test_share_and_changed_read_by_rule():
    lines = [record(label=1, changed="1"), record(label=1, changed="0"), record(), record()]
    (row,) = table(parse(lines), None)
    assert row["probes"] == 4 and row["share"] == pytest.approx(0.5)
    assert row["changed"] == pytest.approx(0.5)
    assert row["thin"]


def test_wilson_contains_the_share_and_narrows_with_n():
    low, high = wilson(20, 1000)
    assert low < 0.02 < high
    wide_low, wide_high = wilson(2, 100)
    assert wide_high - wide_low > high - low


def write(path, lines):
    path.write_text("\n".join(lines) + "\n")
    return str(path)


def test_compare_separates_only_disjoint_intervals(tmp_path):
    honest = write(tmp_path / "h.tsv", [record(rule="futility", label=int(i < 10)) for i in range(1000)])
    planted = write(tmp_path / "p.tsv", [record(rule="futility", label=int(i < 60)) for i in range(1000)])
    same = write(tmp_path / "s.tsv", [record(rule="futility", label=int(i < 12)) for i in range(1000)])
    assert main(["compare", honest, planted, "--rule", "futility"]) == 0
    assert main(["compare", honest, same, "--rule", "futility"]) == 1


def test_compare_reads_only_the_quiet_moves_when_asked(tmp_path):
    # The noisy moves raise in both reads; only the quiet ones differ.
    noisy = [record(label=1, quiet="0") for _ in range(500)]
    honest = write(tmp_path / "h.tsv", noisy + [record(label=int(i < 10)) for i in range(1000)])
    planted = write(tmp_path / "p.tsv", noisy + [record(label=int(i < 60)) for i in range(1000)])
    assert main(["compare", honest, planted, "--rule", "lmr", "--quiet", "quiet"]) == 0
    assert main(["compare", honest, planted, "--rule", "lmr", "--quiet", "noisy"]) == 1


def test_a_bad_quiet_field_is_refused():
    with pytest.raises(ValueError, match="quiet"):
        parse([record(quiet="x")])


def summary(nodes):
    return f"# search nodes {nodes} probes 0 probe_nodes 0"


def test_mantel_haenszel_of_one_stratum_is_the_crude_ratio_with_woolf_variance():
    planted = parse([record(label=int(i < 10)) for i in range(100)]).records
    honest = parse([record(label=int(i < 5)) for i in range(100)]).records
    ratio, low, high = mantel_haenszel(planted, honest)
    assert ratio == pytest.approx(10 * 95 / (90 * 5))
    woolf = math.sqrt(1 / 10 + 1 / 90 + 1 / 5 + 1 / 95)
    assert low == pytest.approx(ratio * math.exp(-1.959964 * woolf))
    assert high == pytest.approx(ratio * math.exp(1.959964 * woolf))


def simpson(tmp_path):
    """Higher in each depth, lower pooled: the plant moved decisions to the
    deep stratum, where wrong decisions are rare."""
    honest = ([record(depth=1, label=int(i < 100)) for i in range(1000)]
              + [record(depth=6, label=int(i < 2)) for i in range(200)])
    planted = ([record(depth=1, label=int(i < 40)) for i in range(200)]
               + [record(depth=6, label=int(i < 40)) for i in range(2000)])
    return (write(tmp_path / "h.tsv", [summary(1000)] + honest),
            write(tmp_path / "p.tsv", [summary(1000)] + planted))


def test_the_plant_read_passes_a_composition_shift_the_crude_read_misses(tmp_path):
    honest, planted = simpson(tmp_path)
    assert main(["compare", honest, planted, "--rule", "lmr"]) == 1
    assert main(["plant", honest, planted, "--rule", "lmr"]) == 0


def test_an_insensitive_label_fails(tmp_path):
    honest = write(tmp_path / "h.tsv", [summary(1000)]
                   + [record(label=int(i % 20 == 0)) for i in range(4000)])
    planted = write(tmp_path / "p.tsv", [summary(1000)]
                    + [record(label=int(i % 20 == 0)) for i in range(8000)])
    assert main(["plant", honest, planted, "--rule", "lmr"]) == 1


def test_a_thin_read_is_unresolved(tmp_path):
    honest = write(tmp_path / "h.tsv", [summary(1000)]
                   + [record(label=int(i < 2)) for i in range(100)])
    planted = write(tmp_path / "p.tsv", [summary(1000)]
                    + [record(label=int(i < 3)) for i in range(150)])
    assert main(["plant", honest, planted, "--rule", "lmr"]) == 2


def test_a_plant_that_barely_fires_more_is_inactive(tmp_path):
    honest = write(tmp_path / "h.tsv", [summary(1000)]
                   + [record(label=int(i < 10)) for i in range(1000)])
    planted = write(tmp_path / "p.tsv", [summary(1000)]
                    + [record(label=int(i < 200)) for i in range(1100)])
    assert main(["plant", honest, planted, "--rule", "lmr"]) == 3


def test_activation_is_per_main_node(tmp_path):
    # Twice the probes over twice the nodes is the same rate: inactive.
    honest = write(tmp_path / "h.tsv", [summary(1000)]
                   + [record(label=int(i < 10)) for i in range(1000)])
    planted = write(tmp_path / "p.tsv", [summary(2000)]
                    + [record(label=int(i < 400)) for i in range(2000)])
    assert main(["plant", honest, planted, "--rule", "lmr"]) == 3
