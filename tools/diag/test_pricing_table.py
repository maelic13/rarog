#!/usr/bin/env python3
"""Tests for pricing_table.py: the parser sums every search's summary line,
refuses a malformed record, and the planted comparison separates only when
the intervals are disjoint."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

from pricing_table import main, parse, table, wilson  # noqa: E402


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
