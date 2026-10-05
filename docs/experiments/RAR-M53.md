# RAR-M53 — B.2.0.1 repository and document restructure, COMPLETE 2026-09-14, documents only

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**B.2.0.1 repository and document restructure, COMPLETE 2026-09-14, documents only.** The nine tickets of `analysis/repository_review_2026-09.md` §5 in commits `02a1f7f` (U1), `d6f4043` (U2), `63ffbcf` (U3), `ce3c7ed` (U5), `4605d6d` (U6), `04898c9` (U7), `b6b492a` (U8, logo part reverted in `34732a2`), `ebe301e` (U9), `c4b3d1d` (U4, after maintainer approval). Checks: line-level preservation scripts for every move (PLAN, EXPERIMENTS), ID-to-row uniqueness over EXPERIMENTS sections 2–8, a stale-citation and uncited-Markdown census for `analysis/`, `git check-ignore` for each storage statement, `check_guide.py` with its dead-path self-test after every commit. Prediction frozen in the review: PLAN ≤ 950, HISTORY ≤ 450 plus a 300–400-line recovered archive, EXPERIMENTS ≤ 1,450 with 183 rows unchanged, AGENTS ≤ 280, PROCESS ≤ 350, GUIDE ≤ 220, 13 records archived, dangling paths in the four current documents 44 → 0, tracked logos 9 → 3.

## Result / disposition

PLAN 1,508 → 857; HISTORY 928 → 475 plus archives of 640 + 1,714 (legacy GUIDE and PLAN) and 545 (Phase-4 tracker) lines; EXPERIMENTS 1,676 → 410 lines with 193 experiment IDs each in exactly one row (was 171 IDs in 172 rows plus 22 prose-only); AGENTS 447 → 301; PROCESS 448 → 426; GUIDE 249 → 216; 13 analyses archived; zero dangling paths in GUIDE, PLAN, PROCESS and AGENTS; tracked logos 9 (U8 reverted). **Calibration:** four review premises were false and were found by the moves' own checks: the legacy tracker was recoverable from the 2.3.1 commit, HISTORY's forward tracker was the pre-renumbering tracker rather than a duplicate, HISTORY had no B.0/B.1 records, and the ledger template was not PROCESS's packet. Sizes missed on HISTORY (new dated records), PROCESS (it gained the template) and AGENTS; the row count prediction counted only tabular rows.

## Conditional lesson and retry trigger

A document restructure needs a preservation check per move, not a size target: every premise that turned out false would have deleted evidence if the review had been executed as written. Logos are user-facing and stay tracked regardless of citation.

## Source

PLAN B.2.0.1; `analysis/repository_review_2026-09.md`; RAR-M52; `docs/archive/PLAN-closed-leaves-2026-09-14.md`; `analysis/ledger_records_2026-09-14.md`
