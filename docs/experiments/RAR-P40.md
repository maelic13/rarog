# RAR-P40 — C.11 evaluation share of search time, against B.9's 24.3% — REGISTERED 2026-10-21, before any capture

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**C.11 evaluation share of search time — REGISTERED 2026-10-21, before any capture.** PLAN's Phase C speed requirement (4): C.11 records the evaluation's share of search time against B.9's, and a share above it is explained in this record, not accepted silently. **B.9's figure:** 24.3% of process samples, B.7.3's re-profile of the B head by the same instrument (archived PLAN, B.7.3); RAR-P27 read 21.2% at B.7.2.

- **Binary:** `analysis/artifacts/c11-checkpoint/profile-bin/rarog.exe` (SHA-256 `9D1E3786D929A64C62B1779CD3B73BFF51200F101E173077F2CFDB5A6EB9CFE0`) and its PDB (`E52E3957…`): `CARGO_PROFILE_RELEASE_DEBUG=2 cargo xtask build --arch pext --pgo` at `ef1f731` (engine source `9fb525e`, rustc 1.99.1), as the B.7 profiles were built; debug information does not change code generation, and it benches **13,987,223 / EBF 2.565**.
- **Capture:** `tools/diag/board_search_profile_etw.ps1 -Exe <that binary>` (five frozen cohorts, 600,000 nodes, 5 repeats, 8 kHz), in an elevated PowerShell 7 as the second step of `analysis/artifacts/c11-checkpoint/run_all.ps1`, output `tools/results/c11-profile-etw`, on an idle host.
- **Reading (mine, afterwards):** `python tools/diag/summarize_board_search_etw.py --scheme search --exe <binary> --symbolizer C:\msys64\clang64\bin\llvm-symbolizer.exe <the five butterfly reports>`: the **evaluation** region (`rarog::eval::`, `rarog::kpk::`, `raw_eval`) as a share of process samples, with evaluation correction, move ordering, search node, TT and the rest beside it, and the share of engine samples resolved (B.7.2 resolved all of them). The symbolizer is MSYS2's LLVM 22 build; the PDB format does not depend on the compiler's LLVM, and the resolved share shows whether it read the symbols.

**Prediction, frozen 2026-10-21 before any capture:** evaluation **28%** of process samples, 80% band **[22%, 36%]**. Basis: Phase C added king-safety, threats and pawn terms and so evaluation work per node; C.12 then made both the evaluation and the rest of the search cheaper, and a faster search raises the evaluation's relative share even at equal evaluation cost. A share above 24.3% is expected and is explained here by the regions that grew. Evidence layer: profile samples, not NPS or Elo.

## Result / disposition

Pending.

## Source

PLAN, Phase C speed requirement (4); archived PLAN B.7.3; RAR-P27 (`analysis/b72_profile_2026-10-01.md`)
