# RAR-P40 — C.11 evaluation share of search time, against B.9's 24.3% — REGISTERED 2026-10-21, before any capture; READ 2026-10-21: 21.1%

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**C.11 evaluation share of search time — REGISTERED 2026-10-21, before any capture.** PLAN's Phase C speed requirement (4): C.11 records the evaluation's share of search time against B.9's, and a share above it is explained in this record, not accepted silently. **B.9's figure:** 24.3% of process samples, B.7.3's re-profile of the B head by the same instrument (archived PLAN, B.7.3); RAR-P27 read 21.2% at B.7.2.

- **Binary:** `analysis/artifacts/c11-checkpoint/profile-bin/rarog.exe` (SHA-256 `9D1E3786D929A64C62B1779CD3B73BFF51200F101E173077F2CFDB5A6EB9CFE0`) and its PDB (`E52E3957…`): `CARGO_PROFILE_RELEASE_DEBUG=2 cargo xtask build --arch pext --pgo` at `ef1f731` (engine source `9fb525e`, rustc 1.99.1), as the B.7 profiles were built; debug information does not change code generation, and it benches **13,987,223 / EBF 2.565**.
- **Capture:** `tools/diag/board_search_profile_etw.ps1 -Exe <that binary>` (five frozen cohorts, 600,000 nodes, 5 repeats, 8 kHz), in an elevated PowerShell 7 as the second step of `analysis/artifacts/c11-checkpoint/run_all.ps1`, output `tools/results/c11-profile-etw`, on an idle host.
- **Reading (mine, afterwards):** `python tools/diag/summarize_board_search_etw.py --scheme search --exe <binary> --symbolizer C:\msys64\clang64\bin\llvm-symbolizer.exe <the five butterfly reports>`: the **evaluation** region (`rarog::eval::`, `rarog::kpk::`, `raw_eval`) as a share of process samples, with evaluation correction, move ordering, search node, TT and the rest beside it, and the share of engine samples resolved (B.7.2 resolved all of them). The symbolizer is MSYS2's LLVM 22 build; the PDB format does not depend on the compiler's LLVM, and the resolved share shows whether it read the symbols.

**Prediction, frozen 2026-10-21 before any capture:** evaluation **28%** of process samples, 80% band **[22%, 36%]**. Basis: Phase C added king-safety, threats and pawn terms and so evaluation work per node; C.12 then made both the evaluation and the rest of the search cheaper, and a faster search raises the evaluation's relative share even at equal evaluation cost. A share above 24.3% is expected and is explained here by the regions that grew. Evidence layer: profile samples, not NPS or Elo.

## Result / disposition

**Captured 2026-10-21 21:08–21:09 local** by `run_all.ps1`'s second step in an elevated shell (`tools/results/c11-profile-etw/`: manifest exe `9d1e3786…`, PDB `e52e3957…`, 600,000 nodes, 5 repeats, 8 kHz; `analysis/artifacts/c11-checkpoint/step-2-profile.log`). All five cohorts were captured and every report passed the tool's own contract checks; the script then failed in its clean-up, restoring the PDB beside the executable from the slot it had itself cleared, which was the PDB's only location here. The PDB was restored from the capture's `symbols/` copy (SHA-256 equal to the registered one) and the tool fixed to restore from that copy (`d42fa46`); the fixed path has not yet run, since it needs an elevated shell.

**Read** (`summarize_board_search_etw.py --scheme search`, MSYS2's `llvm-symbolizer`; `tools/results/c11-profile-etw/summary-search.json`): 169,211 process samples, **every cohort's engine samples resolved (100%)**. **Evaluation 21.06%** of process samples (opening 30.39%, middlegame 19.89%, check-heavy 24.33%, promotion 8.60%, sparse endgame 10.57%); evaluation correction 4.36%; move ordering 22.98%; search node 14.94%; move generation 9.35%; SEE 6.78%; TT 5.83%; make/unmake 5.41%; history 4.73%; check queries 1.46%. **Against B.9's 24.3%: 3.2 points lower**, so PLAN's requirement (explain a share above it) has nothing to explain.

**Prediction calibration, appended after exposure.** Predicted 28% in [22%, 36%]: **a miss in sign and magnitude**, the share fell. The assumption that failed is that Phase C's added terms (king safety, threats, pawns) would outweigh C.12's rewrites; C.12 rewrote the evaluation first (its shared attack-map producer and cache work), and those cut more than the units added. A share is not a cost per node, and this read does not give one.

## Conditional lesson and retry trigger

**The classical evaluation leaves Phase C cheaper in relative terms than it entered: 21% of search time.** Move ordering (23%) is again the largest region. **Retry trigger:** Phase F's NNUE, whose inference cost replaces this region; E.1 may re-profile the final classical head.

## Source

PLAN, Phase C speed requirement (4); archived PLAN B.7.3; RAR-P27 (`analysis/b72_profile_2026-10-01.md`)
