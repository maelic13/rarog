"""Unit tests for the pure parsing halves of fixed_budget_probe.py.

The engine-driving half is exercised by the smoke run recorded in the B.0
tooling commit; these tests pin the EPD and `info` parsing that every screen
threshold in PLAN B.2.2 depends on.
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import fixed_budget_probe as probe  # noqa: E402


def test_parse_epd_reads_wac_and_suite_lines(tmp_path):
    epd = tmp_path / "suite.epd"
    epd.write_text(
        "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - bm Qg6; id \"WAC.001\";\n"
        "rnb1kbnr/ppp1pppp/8/4q3/8/2N5/PPPP1PPP/R1BQKBNR w KQkq - 2 4 ; cohort opening ; src book#0\n"
        "\n"
        "# comment\n"
        "8/7p/5k2/5p2/p1p2P2/Pr1pPK2/1P1R3P/8 b - - bm Rxb2; id \"WAC.002\";\n",
        encoding="utf-8",
    )
    items = probe.parse_epd(str(epd))
    assert [i["id"] for i in items] == ["WAC.001", "pos2", "WAC.002"]
    assert items[0]["bm"] == ["g3g6"], "SAN bm must be converted to UCI"
    assert items[0]["fen"].endswith(" 0 1")
    assert items[1]["bm"] == []
    assert items[1]["fen"] == "rnb1kbnr/ppp1pppp/8/4q3/8/2N5/PPPP1PPP/R1BQKBNR w KQkq - 0 1"
    assert items[2]["bm"] == ["b3b2"]


def test_info_fields_reads_depth_pv_and_multipv():
    line = "info depth 7 seldepth 12 multipv 1 score cp 31 nodes 4321 nps 9 time 5 pv e2e4 e7e5"
    fields = probe.info_fields(line)
    assert fields == {"depth": 7, "seldepth": 12, "nodes": 4321, "time": 5,
                      "pv1": "e2e4", "multipv": 1}
    assert "pv1" not in probe.info_fields("info depth 3 score cp 0 nodes 10 time 1")


def test_a_node_budget_reads_the_last_completed_iteration_not_a_bound_line():
    # phase-4 position 2 on the current head at 300k nodes: depth 13 completes,
    # depth 14 prints only aspiration bound lines before the budget runs out.
    infos = [
        "info depth 13 seldepth 27 multipv 1 score cp 148 nodes 173821 time 106 pv d5d4 c3d5",
        "info depth 14 seldepth 26 multipv 1 score cp 112 upperbound nodes 200255 time 116 pv d5d4",
        "info depth 14 seldepth 31 multipv 1 score cp 84 lowerbound nodes 284077 time 164 pv d5d4",
    ]
    last = probe.last_completed(infos)
    assert (last["depth"], last["nodes"]) == (13, 173821)
    assert probe.last_completed(infos[1:]) == {}, "only bound lines: no completed iteration"
    assert probe.last_completed([]) == {}


def test_engine_spec_carries_uci_options():
    assert probe.parse_engine_spec("core=D:/x/rarog.exe") == ("core", "D:/x/rarog.exe", [])
    assert probe.parse_engine_spec("core=rarog.exe|AblationMask=128|Hash=16") == (
        "core", "rarog.exe", [("AblationMask", "128"), ("Hash", "16")])
    try:
        probe.parse_engine_spec("core=rarog.exe|AblationMask")
    except SystemExit as exit_:
        assert "Name=Value" in str(exit_)
    else:
        raise AssertionError("an option without a value must exit")


def test_option_rejection_is_recognised():
    assert probe.option_rejected("info string No such option: AblationMask\n")
    assert probe.option_rejected("No such option: Foo")
    assert not probe.option_rejected("info depth 3 score cp 0 nodes 10 time 1 pv e2e4")


def test_main_rejects_unknown_mode(capsys):
    try:
        probe.main(["bogus", "10", "x.epd", "out.json", "e=engine.exe"])
    except SystemExit as exit_:
        assert "unknown mode" in str(exit_)
    else:
        raise AssertionError("unknown mode must exit")
