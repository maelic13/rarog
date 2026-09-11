# 4.11.10 conversion-claim correction -- RAR-M24

Status: complete 2026-09-06. This is a records correction and a zero-game
endgame measurement. It does not change engine code or revisit either accepted
SPRT verdict.

## Question and method

RAR-E14 invalidated every v1 pawn-family conversion result, including
RAR-E08's aggregate `83.24% -> 83.45%`, RAR-E12's `0.8345 -> 0.8477`,
RAR-E11's entire reference comparison, and the interpretation of RAR-E08's
KQ-KP `-3.8 pp` result. This step reran the preserved binaries through the
repaired `rarog-endgame-truth-v2` runner instead of deriving a v2 number from
a v1 report.

All runs use Syzygy 3--6, 60,000 nodes/move, a 100-ply limit, seed `6200600`,
16 MB hash, 30 one-thread workers, and `--per-position`. The derivation
asserts equal v2 schema, conditions, cohort digest, family set, FEN/index, and
Syzygy WDL/DTZ before comparing an arm pair.

| Comparison | Binaries | Cohort |
|---|---|---|
| RAR-E08 aggregate | `rarog-pre-e08.exe` / `rarog-e08-head.exe` | 19 families x 100 |
| RAR-E08 focus | same pair | KBN-K, KNN-KP, KQ-KP, KP-KP x 400 |
| RAR-E12 aggregate | `rarog-e08-head.exe` / `rarog-e09cand-pext-pgo.exe` | 19 families x 100 |

`rarog-e09cand-pext-pgo.exe` is the preserved RAR-E12 candidate artifact:
its manifest records the registered `8,044,078 / 2.481` fingerprint. The
E08/E12 comparison measures the complete fitted vectors, not individual terms
inside either refit.

## Corrected results

| Claim | Historical v1 statement | Corrected v2 result | Disposition |
|---|---|---|---|
| RAR-E08 aggregate conversion | `0.8324 -> 0.8345` | **1255/1372 = 0.9147 -> 1254/1372 = 0.9140**; paired 54 pre-only, 53 E08-only | **SUPERSEDED.** The original aggregate did not survive the instrument repair; the corrected full cohort is effectively flat but one conversion lower. |
| RAR-E08 KQ-KP conversion | `390/396 -> 375/396`, -3.8 pp | **same 390/396 -> 375/396, -15, -3.79 pp**; paired 20 pre-only, 5 E08-only | **CONFIRMED under v2.** It remains a historical causal debt for 4.12.13, but it is not evidence of a persistent current-budget deficit: the current head's 60k 94/98 closes to 98/98 at both 200k and 600k. |
| RAR-E12 aggregate conversion | `0.8345 -> 0.8477` | **1254/1372 = 0.9140 -> 1278/1372 = 0.9315**, +24; paired 43 E08-only, 67 E12-only | **SUPERSEDED and restated.** The v1 numbers are not comparable. The complete E12 candidate improves corrected aggregate conversion, without isolating any one HCE term. |
| RAR-E12 KQ-KP “debt repaid” | conversion and DTZ described as improved | conversion **96/98 -> 94/98** while DTZ progress **473/1270 = 0.3724 -> 499/1083 = 0.4608** | **OVERBROAD.** DTZ progress improves, but conversion does not; the RAR-E08 conversion debt was not repaid by this evidence. |
| RAR-E11 reference result | Stockfish 90.2%, worse in three families | reference **1361/1372 = 0.9920**, current head **1276/1372 = 0.9300**, worse in none | **SUPERSEDED in full** by the v2 baseline. |

The other three RAR-E08 400-position focus results also reproduce under v2:
KBN-K `378/398 -> 380/398` (+0.50 pp), KNN-KP `13/77 -> 7/77`
(-7.79 pp), and KP-KP `159/165 -> 154/165` (-3.03 pp). Thus the corrected
instrument changes the invalid aggregate accounting, but it does not erase
the original four-family paired measurements.

RAR-E12's KBN-K DTZ regression is also reproduced by the direct v2 pair:
`2170/2989 = 0.7260 -> 2146/3178 = 0.6753`. Its conversion is `90/98 ->
88/98`. This stays assigned to 4.12.14, but it is a complete-refit result,
not proof that one king-safety term caused it.

## Consequences

- RAR-E08's and RAR-E12's **Elo verdicts stand**. Fastchess, not the truth
  runner, played those games.
- The v1 aggregate conversion claims and every RAR-E11 comparison are
  retained as historical text but marked superseded in place.
- KQ-KP retains its historical RAR-E08 causal debt and belongs to 4.12.13.
  RAR-M21 prevents that leaf from treating the 60k shortfall as a persistent
  deployment defect.
- 4.12.14 retains the corrected KBN-K DTZ target, with the explicit limit that
  the E12-only causal attribution is unisolated.

## Reproduction and evidence

The archived derivation and five byte-preserved reports are
`analysis/artifacts/conversion-claims-correction-20260906.zip`, SHA-256
`1D45B49AB167D45B97836B7EEEAC69B5946EF227BFF6C51B8C224B7A2FA1E6EF`.
Extract the archive and run `python reproduce.py`; it loads the bundled reports
by relative path, asserts the paired contracts, then prints all totals and pair
matrices. This archive supplies the recipe and inputs without depending on a
branch or an ignored results directory.
