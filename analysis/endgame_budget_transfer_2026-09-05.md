# 4.11.7 budget transfer — RAR-M21

Status: registered before new conversion runs on 2026-09-05; completed and
reconciled on 2026-09-06. All six runs exited successfully.
The maintainer explicitly authorized agent-run heavy computation for this
session. This resumes the existing leaf; it does not create a new strength gate.

## Frozen question and scope

Repeat the entire corrected v2 cohort: 19 families, 100 positions each,
seed 6200600, cohort SHA-256
`fe4866045506636f884ee30526b4188c3def9ca9747f5960ea5c5e7cba5dbb5e`.
Budgets are 60,000, 200,000 and 600,000 nodes per move, maximum 100 plies,
Hash 16 MiB, Threads 1, engine tablebases disabled, external Syzygy 3–6-man
truth, persistent TT within each game and reset between games, 30 workers.
Use the unchanged `endgame_budget_bracket.py` / `endgame_truth.py` instrument,
per-position output, sequential engine arms. No reuse of existing reports.

Rarog source is `6e8044a` on dev, rebuilt with no Cargo features,
`RUSTFLAGS=-C target-cpu=native --cfg rarog_pext`, release, locked, no PGO.
The isolated build reproduces bench 13: **6,901,489 / EBF 2.458**.
The reference is the existing Stockfish 18 BMI2 executable used by the v2
study, now frozen by binary hash. It is an external binary, not a claim to
have rebuilt today's Stockfish checkout. Archive both executables and record
hashes, toolchain, harness hashes, commands and return codes in the manifest.

## Interpretation fixed before results

Check the fresh 60k arm against its historical v2 report before proceeding
to larger budgets. Any changed cohort or outcome is a confound to investigate,
not permission to silently replace the baseline. Compare converted / initially
theoretically won positions by family at every budget, plus paired per-position
success changes. Preserve all families, including unfavorable or null results.

The decisive existing claims are the KQ-KR (23/100), KBN-K (10/98),
KNN-KP (9/23), KR-KP (8/98), KRP-KR (5/73), KRP-KB (5/96) and
KQ-KP (4/98) conversion deficits against the reference at 60k, together with
the aggregate 85/1372 deficit. Report whether each deficit persists, shrinks,
disappears or reverses across the bracket; differences need not be monotone.
The full cohort also guards against overlooking smaller deficits or losses.

This is a diagnostic transfer study on an already inspected cohort, not
independent held-out confirmation, candidate fitting, an Elo estimate, or
evidence of theoretical optimality. Reference success is attainable evidence,
not a mathematical ceiling. Fixed-node engine budgets are not equal time.
Only the accepted head and reference are bracketed. This does not bracket
historical refit or mate-drive off/on arms: full conversion by today's head
at a higher budget cannot erase a causal debt measured between two historical
arms at 60k. Those comparisons retain their explicit budget and owners at
4.11.9/4.11.10 and the relevant 4.12 family steps.
Static drawn-share overclaims and initial-position tablebase truth are separate
measurements; do not describe them as conversion-budget findings. A conclusion
restricted to one budget remains explicitly provisional. Keep 4.12 ranking v2
as the historical prioritization input and assign any needed revision explicitly.

## Completed results

Both fresh 60k arms reproduce their historical v2 family reports exactly,
including all per-position records. All six reports have identical ordered
FENs, initial WDL/DTZ labels and cohort hashes. Conversion totals were checked
against the individual initially won positions, not inferred from percentages.
There are 11,400 endgame playouts across six runs; these are not strength games.

Cells below are converted positions, Rarog / Stockfish. The denominator is
the same initially theoretically won cohort for every budget and engine.

| Family | Won | R / SF 60k | R / SF 200k | R / SF 600k |
|---|---:|---:|---:|---:|
| KQ-K | 100 | 98 / 100 | 100 / 100 | 100 / 100 |
| KR-K | 100 | 96 / 100 | 100 / 100 | 100 / 100 |
| KBB-K | 100 | 100 / 100 | 100 / 100 | 100 / 100 |
| KBN-K | 98 | 88 / 98 | 98 / 98 | 98 / 98 |
| KNN-K | 1 | 1 / 1 | 1 / 1 | 1 / 1 |
| KP-K | 80 | 77 / 80 | 80 / 80 | 80 / 80 |
| KPP-K | 98 | 96 / 98 | 98 / 98 | 98 / 98 |
| KBP-K | 94 | 94 / 94 | 94 / 94 | 94 / 94 |
| KR-KP | 98 | 90 / 98 | 97 / 98 | 98 / 98 |
| KR-KB | 28 | 26 / 28 | 28 / 28 | 28 / 28 |
| KR-KN | 45 | 42 / 45 | 44 / 45 | 45 / 45 |
| KQ-KP | 98 | 94 / 98 | 98 / 98 | 98 / 98 |
| KQ-KR | 100 | 77 / 100 | 87 / 100 | 97 / 100 |
| KNN-KP | 23 | 5 / 14 | 9 / 15 | 6 / 14 |
| KRP-KR | 73 | 67 / 72 | 70 / 72 | 71 / 72 |
| KRP-KB | 96 | 90 / 95 | 95 / 96 | 95 / 96 |
| KBP-KB | 26 | 24 / 26 | 26 / 26 | 26 / 26 |
| KBP-KN | 57 | 55 / 57 | 55 / 57 | 55 / 57 |
| KP-KP | 57 | 56 / 57 | 56 / 57 | 56 / 57 |
| **Total** | **1372** | **1276 / 1361** | **1336 / 1363** | **1346 / 1362** |

The aggregate net deficit is **85 / 27 / 16 positions**, not 85 at every
budget. At 60k/200k/600k respectively, reference-only successes are
88/30/17 and Rarog-only successes are 3/3/1. Net deficits therefore do not
identify all individual positions where the reference demonstrates a conversion.

## Which conclusions transfer, and their owners

- **KBN-K and KQ-KP conversion shortfalls do not transfer**: both reach full
  conversion at 200k and 600k. Keep their 60k result as a budget-specific
  diagnostic. Do not claim a persistent conversion deficit to justify new
  terms. KBN-K's DTZ progress is 2146/3178 (67.53%), 2152/2749 (78.28%) and
  2137/2704 (79.03%); changing move denominators make this descriptive evidence,
  not a same-budget regression-floor comparison. Historical refit debts remain
  owned by 4.11.10 and 4.12.13/4.12.14 pending matched-arm adjudication.
- **KR-KP's shortfall is budget-sensitive**: 8, 1, then zero positions behind
  the reference. KR-KN also reaches full conversion at 600k. Their static draw
  overclaims are separate evidence and remain owned by 4.12.8 and 4.12.4.
- **KQ-KR's deficit persists but shrinks sharply**, 23/13/3 positions (4.12.10).
  The old 23-position figure is a 60k observation, not a deployment-wide fact.
- **KNN-KP, KRP-KR and KRP-KB remain behind at every budget**: net deficits
  9/6/8, 5/2/1 and 5/1/1 respectively. Retain investigation at 4.12.15,
  4.12.2 and 4.12.6, without reading these small cohorts as precise population
  effect sizes. KNN-KP is non-monotone in both engines; more nodes do not
  monotonically improve this fixed 100-ply conversion instrument.
- **KBP-KN and KP-KP retain smaller deficits**, 2 and 1 at every budget
  (4.12.9 and 4.12.12). KQ-K, KR-K, KP-K, KPP-K, KR-KB and KBP-KB reach
  full conversion at both larger budgets. KBB-K, KNN-K and KBP-K already did
  so at 60k; KNN-K has only ONE theoretically won start, not broad coverage.

Paired gains/losses also matter: from 60k to 200k Rarog gains 70 conversions
and loses 10; from 200k to 600k it gains 19 and loses 9. Stockfish gains/loses
3/1 and then 1/2. `comparison.json` preserves the exact gained/lost FENs for
both transitions and engines. Each family owner should review those cases
before attributing a residual to evaluation, search or the playout horizon;
search-mechanism follow-up belongs to 4.15.1 if the family analysis establishes
that dependency. This study does not diagnose the cause.

**Roadmap disposition:** complete 4.11.7. Keep ranking v2 as its frozen 60k
historical input; do not overwrite it or select a more favorable budget after
seeing results. 4.11b.18 must use this bracket when refreshing affected evidence,
and 4.12.1 must reconcile any successor ranking with it. Existing static-draw
priorities and historical causal debts are not cancelled. This resolves the
scheduling hold without dropping any later leaf. Next: 4.11.8 label audit.

## Verification and reproducible evidence

Debug and release `cargo test --all-features` passed in the isolated
`D:/chess/results/rarog-4117-checks` target directory; `cargo fmt --check`,
`cargo clippy --all-features --all-targets -- -D warnings`, and all 156 Python
tooling tests passed. The measured production binary has no Cargo features,
is archived separately, and its hash is checked before each arm. The result
validator also demonstrably refuses an incomplete study and swapped FEN
identities even when cohort metadata and aggregate conversion totals match.
No engine source,
parameters, search policy or evaluation was changed; no SPRT was run.

`analysis/artifacts/budget-transfer-20260905.zip` preserves exact raw bytes:
six reports, both historical 60k reports, run logs, manifest, paired comparison,
the frozen measurement driver/harness and verification logs. ZIP protects the
recorded hashes from Git newline conversion. The archive builder checks every
member byte-for-byte against its source. Binary archives remain at
`D:/chess/results/budget-transfer-20260905/{head,reference}.exe`; their hashes,
the exact toolchain, build and executed argument lists are in the manifest.
Per-arm `bracket.json` files contain only their most recent single-budget
invocation; use the six raw reports and `comparison.json` for the full bracket.

For inspection, extract the ZIP to an empty scratch directory. For a fresh
rerun, use the registered source and matching external reference/TB inputs,
execute the manifest's production build first, and verify bench 13. The driver
refuses existing output/archive directories to protect the measured artifacts.
It requires the two historical reports at the paths preserved in the ZIP.

```powershell
python tools/diag/run_4117_registered.py
python tools/diag/summarize_4117.py
python tools/diag/archive_4117.py
```
