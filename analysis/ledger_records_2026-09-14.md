# Ledger records moved out of EXPERIMENTS.md, 2026-09-14

Moved verbatim on 2026-09-14 (PLAN B.2.0.1) so that every experiment in
`EXPERIMENTS.md` is one row. Each section below is a record, registration or
note that the ledger once held as prose; the row with the same ID cites it.
Nothing was reworded: numbers, dates, IDs and verdicts are as recorded, and
retired step numbers and paths are left as they were written.

## RAR-M21 (Measurement, harness and tuning)

**RAR-M21 — 4.11.7 budget transfer, registered 2026-09-05; COMPLETE 2026-09-06.**
Baseline `6e8044a`, exact production features (empty), bench 13
6,901,489 / EBF 2.458. Frozen Stockfish 18 reference; full corrected 19-family
cohort, 100 positions/family, seed 6200600, 60k/200k/600k nodes/move,
100-ply cap, Hash 16, Threads 1, engine TB disabled, 30 workers.
Hypothesis: the existing per-family conversion deficits persist at deployment-
representative budgets. This is a diagnostic, not an SPRT or an acceptance
gate; no engine change is proposed. Stop on failed commands, changed cohort,
or failure to reproduce either historical 60k family report; investigate
before advancing. Complete all registered budgets otherwise. Protocol and
decisive cases: `analysis/endgame_budget_transfer_2026-09-05.md`.
Exact commands, binary/harness hashes and exit statuses:
`tools/results/budget-transfer-20260905/manifest.json`, preserved with all raw
reports in `analysis/artifacts/budget-transfer-20260905.zip`;
reproduction driver: `python tools/diag/run_4117_registered.py` (build/input
prerequisites and protected output paths are documented in the analysis).
**Result:** both fresh 60k reports reproduce exactly; all six commands succeed.
Rarog converts **1276/1336/1346 of 1372** at 60k/200k/600k, Stockfish
**1361/1363/1362**: net deficit **85/27/16**. KBN-K and KQ-KP reach full
conversion at both higher budgets; a persistent conversion-defect claim from
their 60k result is not supported. KQ-KR remains behind **23/13/3**;
KNN-KP **9/6/8**, non-monotone. KRP-KR, KRP-KB, KBP-KN and KP-KP also
retain deficits across the bracket. Paired Rarog gains/losses are **70/10**
then **19/9**, so aggregate improvement does not imply per-position dominance.
**Disposition:** close 4.11.7; preserve v2's frozen 60k ranking, attach this
budget qualification to 4.11b.18/4.12.1 and the family owners. Static-draw
overclaims and historical matched-arm refit/mate-drive debts are not cancelled.
No engine implementation or strength gate. Debug/release tests, fmt, Clippy,
156 tooling tests and report/byte-level archive validation passed.

## RAR-M22 (Measurement, harness and tuning)

**RAR-M22 — 4.11.8 datagen label audit, COMPLETE 2026-09-06.** Hash-verified
the two 8,000-node `hce-v2` PGN segments (600,000 games total) and the
8,000-node `hce-v3` source PGN of `hce-v3-tb` (602,619 games), then audited
each game's first 3–6-man Syzygy clean win against its final PGN result. Cursed
wins excluded; only the first clean win per game counted. **Result:** hce-v2
has 26,316 not won / 134,948 clean wins (**19.50%**, **4.39% of all games**);
hce-v3 source 54,186 / 266,490 (**20.33%**, **8.99% of all games**). Both
budgets are 8,000, so no budget comparison is available. This is game-level
raw-label evidence, not a remaining row-error count: `hce-v3-tb` already
Syzygy-corrected 125,643 ≤6-man CSV rows and still leaves >6-man rows unchanged.
**Disposition:** the raw-game labels are materially biased toward draws;
4.13.1 owns row-level lineage and separate post-hoc/whole-game contracts. No
refit, engine change or strength claim. Full analysis and byte-preserved output:
`analysis/datagen_label_audit_2026-09-06.md`,
`analysis/artifacts/datagen-label-audit-20260906.zip`.

## RAR-M23 (Measurement, harness and tuning)

**RAR-M23 — 4.11.9 mate-drive promotion closure, COMPLETE 2026-09-06.**
Recompared the hash-pinned pre-4.9a.4 and accepted-mate-drive reports on their
identical 19-family, 100-position-per-family, 60,000-node, seed-6200600 cohort;
the derivation asserts equal schema, Syzygy path, budget, seed, family set,
FEN, index and Syzygy truth before comparing `mated` conversion. Six families
change: direct KBB-K +22 and KBN-K +76; promotion-reached KPP-K net 0, KBP-K
net +2, KBP-KB net **-1**, KBP-KN net **-1**. **Disposition:** the two
negative-net families are causal debt owned by 4.12.7/4.12.9. Their paired
gains do not cancel a family loss; nonnegative-net KPP-K/KBP-K remain closure
guards. The reports use the pre-4.10 material-shed instrument, so this is not
a current conversion floor or a rollback case. No engine change, game or
strength claim. Full matrix and byte-preserved inputs/derivation:
`analysis/mate_drive_promotion_closure_2026-09-06.md`,
`analysis/artifacts/mate-drive-promotion-closure-20260906.zip`.

## RAR-M24 (Measurement, harness and tuning)

**RAR-M24 — 4.11.10 corrected conversion claims, COMPLETE 2026-09-06.**
Reran the preserved RAR-E08 baseline/head and RAR-E08/E12-candidate binaries
through the repaired v2 truth runner, requiring matching schema, conditions,
cohort, FEN/index and Syzygy truth before every difference. **RAR-E08's v1
aggregate 83.24% -> 83.45% is superseded:** the corrected pair is
**1255/1372 = 0.9147 -> 1254/1372 = 0.9140**. Its four-family 400-position
result is retained under v2, including **KQ-KP 390/396 -> 375/396, -3.79 pp**;
that is a real historical causal debt for 4.12.13. RAR-M21 qualifies it: the
current head's KQ-KP 60k shortfall closes at 200k and 600k. **RAR-E12's v1
0.8345 -> 0.8477 is superseded:** corrected aggregate conversion is
**1254/1372 = 0.9140 -> 1278/1372 = 0.9315** (+24). KQ-KP DTZ progress rises,
but its conversion falls 96/98 -> 94/98, so the former “debt repaid” statement
was overbroad. RAR-E11 stays superseded in full: corrected reference is
1361/1372 = 0.9920, current head 1276/1372 = 0.9300, and reference is worse in
no family. No engine change or strength claim; both accepted Elo verdicts
remain valid. Full derivation and byte-preserved reports:
`analysis/conversion_claims_correction_2026-09-06.md`,
`analysis/artifacts/conversion-claims-correction-20260906.zip`.

## RAR-M25 (Measurement, harness and tuning)

**RAR-M25 — 4.11b.2 board-v2 instrument and correctness corpus, COMPLETE
2026-09-06.** Added a versioned ten-position profile generated and checked by
`python-chess` 1.11.2. It mechanically requires real single/double checks and
evasions, legal/pinned-illegal EP, quiet/capture underpromotions, all four
castles, and sparse long-history material. Rarog now checks exact canonical
FEN, sorted legal/capture UCI identities, perft/divides, keys, occupancy,
pieces and full restoration through normal/hinted/staged/null/clone/unwind
paths. A negative-control test corrupts a legal move, perft count and board
state and requires the preflight to reject each. Coordinate-ray coverage
passes every relevant slider occupancy on magic and PEXT, debug and release.
The independent `rarog-board-v2` benchmark is not the frozen cross-engine-v1
benchmark: it reports separately precomputed generation, mutation and SEE
workloads, raw samples, a portable `black_box` output checksum, and a warmed
allocation guard that observed zero allocations. The archived magic run has
the compiler, host, flags, input hashes and raw samples at
`analysis/artifacts/board-v2-20260906/`. **No engine change, cross-engine
comparison, NPS conclusion or strength claim.**

## RAR-M26 (Measurement, harness and tuning)

**RAR-M26 — 4.11b.3 parser and fullmove boundaries, COMPLETE 2026-09-06.**
The previous `Move::from_uci("aé1")` passed its four-byte length check and
panicked when slicing through UTF-8. It now refuses every non-ASCII token
before indexing; an actual release UCI `position startpos moves aé1` exits 1
with the existing `CRITICAL ERROR` diagnostic, rather than aborting. Fullmove
is intentionally bounded to `u16`: FEN `0` remains compatible and normalizes
to 1, `65535` is accepted and `65536` is rejected. At the maximum, real and
null black moves saturate, white moves retain the counter, and both undo paths
restore the original FEN/state. The new tests failed first on the debug
overflow and UTF-8 panic, then passed in debug and release; full suites,
fmt and all-feature/all-target clippy pass. A rebuilt default-feature
production `bench 13` is exactly **6,901,489 / EBF 2.458**. This is a
correctness repair for malformed and extreme input, not an NPS or strength
claim.

## RAR-M19 (Measurement, harness and tuning)

**RAR-M19 ownership update, 2026-09-05:** its historical result above is
unchanged. Behavior-neutral value injection and initial normalized SEE
benchmark restoration now belong to **4.11b.6**. Post-final-HCE scale audit
and fitted production policy remain **4.15.3–4.15.4**; **4.15.5** revalidates
the normalized benchmark after fitting. Exposing a benchmark input does not
itself authorize production tuning or SPSA.

## RAR-M27 (Measurement, harness and tuning)

**RAR-M27 — 4.11b.4 SEE contracts and independent fixtures, COMPLETE
2026-09-06.** Baseline `6d1a670`, engine `a170f8c`; no production changes.
Ten threshold calls (two diag-only) and one full-SEE call inventoried with
their ordering/pruning/LMR/history consumers. Eighteen python-chess legal
same-square capture-tree fixtures, independently hand-scored, expose three
debts: king-after-pawn -300 vs current -400/true, newly created pin +100 vs
-230/false, recapture promotion -800 vs 0/true (booleans at threshold zero).
All three named pending acceptance tests fail in debug and release; repair
owner 4.11b.5. Ordinary quiet/quiet-promotion immediate-gain policies remain
explicit, with quiet-aware handling only for ordinary quiets; quiet promotions
have no current production SEE caller. Scoped Rust checks pass 8 tests per
profile, with the 3 debt tests intentionally ignored; five Python oracle
checks, fmt and all-feature/all-target clippy pass. No games or speed claim.
Exact FENs, arithmetic, caller contracts, raw observations and reproduction:
`analysis/see_contract_2026-09-06.md`, `tests/data/see-contract-v1.tsv`,
`analysis/artifacts/see-contract-20260906/`.
**Defect status SUPERSEDED by RAR-M28:** the three failures above are historical
baseline observations; all three acceptance tests are now active and passing.

## RAR-M28 (Measurement, harness and tuning)

**RAR-M28 — 4.11b.5 SEE legality and promotion repair, COMPLETE 2026-09-07.**
Engine/test `fce0b44`, entry `e954e38`. Current-occupancy king safety replaces
stale pin masks; king captures terminate legally, recapture promotions include
the promotion gain and promoted victim value, and threshold comparisons preserve
equality. Values remain 100/320/330/500/900/20000; quiet/promotion shortcut
policies remain explicit. The three repaired full/threshold-zero results are
**-300/false, +100/true, -800/false**. Forty-one independent legal-tree fixtures
and 1,802 legal-capture parity checks pass. Full suites pass **268 debug / 269
release**, zero failures/ignores; six Python tests, fmt and Clippy pass.
Exact-feature production bench is **7,601,220 / EBF 2.474**, +10.14% nodes
against 6,901,489 / 2.458. Development fingerprint updated; **no strength or
comparative NPS claim**, no games. Playing qualification remains 4.11b.17.
The first process-argument bench invocation was a no-op, detected from its
missing summary; it is explicitly invalidated and replaced by a hash-verified
UCI-driver run. Reproduction, source/binary identity, raw logs and exact engine
diff against the entry source: `analysis/see_repair_2026-09-06.md` and
`analysis/artifacts/see-repair-20260906/`.

## RAR-M29 (Measurement, harness and tuning)

**RAR-M29 — 4.11b.6 neutral SEE injection and normalized timing, COMPLETE
2026-09-07.** Engine/test `46f1af2`, entry `2c59911`. `SeeValues` owns the
board scale; production remains 100/320/330/500/900/20000 with no runtime
engine option. Explicit production and normalized 100/300/300/500/900/20000
injection each pass all 41 independent fixtures. Complete suites pass **270
debug / 271 release**, zero failed/ignored; seven oracle tests, fmt and Clippy
pass. Exact production `bench 13` remains **7,601,220 / EBF 2.474**.

The benchmark's deliberately absurd rook=1 input flips its independent probe
false -> true, proving the wire. All three adapters report identical normalized
values and ten move/verdict answers. Three cyclic rounds give threshold-SEE
medians Rarog/Basilisk/Reckless **44.923/58.335/40.823 M captures/s**: Basilisk
is +29.86% over Rarog and Rarog +10.04% over Reckless by median. Rarog's round
span is **12.20%** and it loses the third round to Reckless, so magnitudes are
directional; all timed host-busy checks pass. RAR-M20's native SEE row is
**SUPERSEDED for ranking**, not deleted. Its vectors differed and Rarog's
kernel changed at 4.11b.5, so it is not an injection-overhead baseline.

## RAR-M19 (Measurement, harness and tuning, part 2)

**RAR-M19/RAR-M27/RAR-M28 correction:** board SEE's king sentinel was already
20,000, not `MATE_SCORE`/32,000; those records conflated it with eval's separate
piece-value vector. Kings are never legal SEE victims, so no exchange result
changes. No value fit, games, NPS or Elo claim. Evidence:
`analysis/see_value_injection_2026-09-07.md` and
`analysis/artifacts/see-normalized-20260907/`. Production fitting remains
4.15.3–4.15.4. At this closure point, 4.11b.7 was the next leaf.

## RAR-M30 (Measurement, harness and tuning)

**RAR-M30 — 4.11b.7 full-search board profile, COMPLETE 2026-09-07.** Source
`02420dc`, 20 frozen roots in five cohorts, 600,000 nodes, three counter and
five ETW repeats. Production SHA-256 `3c81ef95...bf1d904dfd0`; diagnostic
`aaeda618...25d42e1`; all **60/60** instrumentation-off searches match depth,
seldepth, reported nodes, score type/value and best move; PV and ponder move
were not compared. All 151,142 engine samples resolve from the archived PE/PDB.
Weighted process shares are generation/legality **6.751%**,
make/unmake **7.143%**, check queries **5.177%**, SEE **5.304%**; relocation
helpers are an overlapping **2.998%**, king lookup **0.544%**. Over 30,604,224
diagnostic nodes, checked makes are 89.02% of real makes, threshold SEE is
92.77% of SEE calls, and 25,718,154 history pushes cause **zero growth**.
No games, NPS acceptance, or strength claim. At closure, 4.11b.8 was next; evidence,
full hashes, time budget and reproduction:
`analysis/board_search_profile_2026-09-07.md` and
`analysis/artifacts/board-search-profile-20260907/summary.json`.

## RAR-M31 (Measurement, harness and tuning)

**RAR-M31 — 4.11b.8 pin discovery measurement, recorded 2026-09-07;
research disposition CLOSED: candidate withdrawn in `c44608a`.**
Baseline `407de51`, engine `2ea279f`; replace four x-ray slider lookups by two
empty-board lookups and test all occupied squares between king and aligned
enemy slider. Keep a sole friendly blocker as pinned. Local board-v2 median
gains over three alternating rounds: legal **+8.54%**, capture **+11.43%**,
staged **+7.41%**. Twelve alternating full-search pairs on each backend measure
generic **+0.57%** (bootstrap 95% interval −0.51% to +1.00%) and PEXT **+1.45%**
(−1.57% to +4.26%); **neither establishes a whole-search gain**. PEXT host load
varied more. Non-PGO, 1T, Hash 16 MiB, frozen 20 roots, 600k node limit.
All four production fingerprints are **7,601,220 / EBF 2.474**; 480 paired
root answers match including PV and ponder. Independent pin-ray oracle,
debug/release suites, PEXT board tests, fmt and Clippy pass. Retained for local
generation gains under the original execution contract; no games or strength
acceptance. The later `b592b40` research card requires a prospective practical
whole-search floor, which this run did not register. It does not qualify the
leaf under that new contract. **Original retention decision SUPERSEDED:**
restore the prior x-ray algorithm and retain its independent oracle. Decline
another standalone campaign before shared-geometry research; this is a research
prioritization decision, not a statistical finding of no gain or a post-hoc
floor. Later cache/search changes were not measured in the timing study.
Restoration against `b90232b`: 274 debug / 275 release tests, fmt and Clippy
pass; fresh no-feature before/after builds reproduce 7,601,220 / EBF 2.474;
20 roots match standard harness identity fields (not full PV/ponder). No new
performance or Elo claim. 4.11b.10 owns any justified, prospectively registered retry.
Playing gate remains 4.11b.17.
Recipe, hashes and raw observations: `analysis/movegen_2026-09-07.md` and
`analysis/artifacts/movegen-20260907/`.

## RAR-M32 (Measurement, harness and tuning)

**RAR-M32 — 4.11b.9 fused ordinary relocation, 2026-09-07; VOID, SUPERSEDED BY
RAR-M33. Its `NO_CHANGE` disposition is WITHDRAWN.** The run was taken while a
Manta SPRT held the host at **50.2–53.4% CPU busy** per arm against 3.7–5.8% for
the comparable 4.11b.8 run; re-measured idle, the same baseline code runs at
3,071,903 nps versus this run's 2,182,590 nps. The full-search timing conclusion
is withdrawn; the deterministic findings below (fingerprint parity, 240 paired
root answers, emitted-code comparison) stand. Original record follows.
**RAR-M32 original text —** Baseline `af83abf` on
`dev`; qualification frozen in `86e39f8` **before** any timing. Candidate fuses
ordinary `QUIET` make/unmake relocation into one from/to mask and one paired
key across mailbox, piece/colour occupancy, `all_occ` and the pawn/minor/
non-pawn keys; captures, double pushes, en passant, promotion, castling and
null moves keep their existing paths. Semantics were exact: both no-feature
builds reproduce **7,601,220 / EBF 2.474** (asserted in-runner before timing),
and **240 paired root answers** (12 pairs x 20 roots) match on name, repeat,
depth, seldepth, reported nodes, score type/value, best move, **full PV and
ponder**. Executables are distinct — baseline `fde1ed0e...bf59a4` (the
registered hash), candidate `0da54ca9...cfd9dcf3`, board arms `72e8be2c...`
and `2166a33e...`; frozen suite `0c8cefdf...6b153e3`. Isolated `make/unmake
only` gained **+16.28% / +15.21% / +15.27%** over three alternating board-v2
rounds, meeting the registered local condition; unchanged noise-control columns
moved by mixed sign and smaller magnitude. Twelve alternating full-search pairs
(600,000 nodes, one discarded warm-up per arm, seed 4119, all pairs retained)
measure a **+1.016%** median, bootstrap 95% **-0.450% to +3.609%**, 10/12
candidate-faster, max host CPU busy 53.43%. **The interval includes zero, so
the frozen retention rule rejects.** Emitted-code screen: `make_move_inner`
**468 -> 568** instructions (+21.4%), whole-crate 87,294 -> 87,994 (+0.80%),
symbol count unchanged — refuting "LLVM already fuses this" while supporting
"larger code repays part of the saving". RAR-M30's 7.143% make/unmake share
projects the primitive gain to +0.96% whole-search versus the measured +1.02%,
so the mechanism behaved as predicted and the miss is **instrument power**, not
mechanism: twelve pairs cannot resolve a ~1% effect. This is insufficient
evidence of deployable value, **not** proof of zero benefit, regression or
defect. No games, NPS acceptance or Elo claim. `src/` restored byte-identical
to `af83abf`; the targeted per-piece-class relocation test is retained in
`8a73cfd`. Closure on the restored tree: fmt exit 0, **275 debug / 276 release**
tests pass, Clippy `--all-features --all-targets` zero warnings. Retry is not
authorized standalone; it belongs to **4.11b.16** under a pooled-PGO build with
a precision calculation and whole-search floor registered before the run.
Playing gate remains 4.11b.17. At this closure point, 4.11b.10 is next.
Recipe, hashes and raw observations: `analysis/relocation_2026-09-07.md` and
`tools/results/relocation-411b9/` (ignored, local).

## RAR-M33 (Measurement, harness and tuning)

**RAR-M33 — 4.11b.9 fused ordinary relocation re-measured on a verified-idle
host, COMPLETE 2026-09-07; ACCEPTED and integrated in `5c439da`.** Baseline
`1d720af` on `dev`; contract frozen in `tools/results/relocation-411b9-v2/
registration.md` and reproduced in the analysis document **before** the
candidate was compiled. Candidate re-implemented from the PLAN handoff because
RAR-M32 saved no patch; scope is **`flags == QUIET` only**, adding
`Board::move_piece` to update both mailbox endpoints, the piece and colour
occupancies, `all_occ` and the applicable pawn/minor/non-pawn keys with one
from/to mask and one paired key, with captures, double pushes, en passant,
promotions, castling and null moves untouched and the position hash still
caller-owned. Prospective prediction: full-search median **+0.7% to +1.2%**
from RAR-M30's 7.143% make/unmake weight and the +15.27% primitive gain
(projection +0.96%); isolated gain +14% to +17%; bootstrap half-width 0.33% to
0.46%. Instrument **32 alternating pairs at 1,200,000 nodes**, one discarded
warm-up per arm, seed 4119, all pairs retained, non-PGO, 1T, Hash 16 MiB,
frozen 20-root suite. The runner now **asserts** host idleness instead of
annotating it, and the gate was proven live by aborting under a deliberately
absurd `0.0` threshold; the gate was recalibrated 10% -> 15% before any timing
against a measured 5.52% ambient on a 32-thread host, changing an
instrument-validity precondition only and leaving the acceptance rule frozen.
Results: both builds fingerprint **7,601,220 / EBF 2.474**; **640/640** paired
root answers match on depth, seldepth, reported nodes, score type/value, best
move, **full PV and ponder**; host busy min 5.41 / mean 6.67 / **max 11.80%**;
isolated `make/unmake only` **+16.33 / +17.30 / +19.32%** across three
alternating rounds; full-search median **+0.876%** (3,071,903 -> 3,098,821 nps),
95% bootstrap **[+0.050%, +2.055%]**, 23/32 candidate-faster. **The interval
excludes zero, so the frozen rule retains the candidate.** Emitted code
`make_move_inner` **468 -> 542** instructions; RAR-M32's archived candidate was
568, so this is the same mechanism with leaner codegen, reported as
corroboration rather than exact reproduction. Artifacts distinct: baseline
`62ac2599...`, candidate `d364f2ad...`, board arms `afd11222...` and
`e20b865a...`; the baseline `.s` hashes identically to RAR-M32's
(`75ed0249...`), proving the baseline source state reproduced exactly.
Calibration: magnitude HIT (+0.876% against a predicted +0.7–1.2%), isolated
gain HIT but slightly under-predicted (one round at +19.32%), **interval width
MISS** — actual half-width **1.003%** against a predicted 0.33–0.46%, because
the projection scaled a bootstrapped median's variance by naive `sqrt(n)`.
A post-hoc 200-seed sweep excludes zero in **198/200** (lower bound -0.017% to
+0.101%), characterising robustness without altering the frozen verdict; the
evidence supports "the gain is above approximately zero", not a bankable floor
of 0.9%. Behaviour-neutral, so no game gate is owed and **no Elo is claimed**;
cluster playing qualification remains 4.11b.17. Debug 275 / release 276 tests,
fmt and Clippy `--all-features --all-targets` clean; a fresh no-feature build of
the committed source reproduces 7,601,220 / EBF 2.474. At this closure point,
4.11b.10 is next. Recipe, hashes and raw observations:
`analysis/relocation_2026-09-07.md` and `tools/results/relocation-411b9-v2/`
(ignored, local; `candidate.patch` archived there).

## RAR-M34 (Measurement, harness and tuning)

**RAR-M34 — 4.11b.10 shared pin/check information, COMPLETE 2026-09-08;
research disposition `NO_CHANGE`.** Source `33c373c` on `dev`. Closed on
**structure, not cost**: the three producers share no work. `compute_pinned`
queries from our king against their sliders, `check_info` from their king
against our sliders — different square, different piece sets, only `all_occ`
in common — so no cache of either can serve the other in any node.
`see_recapturer` queries `attackers_to_color(king, after, !side)` against the
**evolving** exchange occupancy, so reusing a real-position pin or attack mask
is the stale `see_pins` defect repaired at 4.11b.5: a correctness boundary, not
a tradeoff. The cross-ply candidate (parent `check_info` versus child
`compute_pinned` at the same king square after the side flip) fails on changed
occupancy and on a different predicate — either-colour sole blocker versus
sole friendly blocker behind an x-ray. The one real sharing opportunity, one
pinned set per node across capture and quiet stages, was already delivered by
the 10.3 speed pass and is measurably active: **422,246** staged quiet
generations cost **zero** extra `compute_pinned` calls. Exact counters
(`bench 13`, `RAROG_DIAG_SAMPLE_STRIDE=1`, diag build `6b6c3e18...`, nodes
**7,601,220** unchanged so the instrument does not perturb the search):
`board_see_threshold_calls` 7,547,296 (**0.993/node**),
`board_gives_check_fast_calls` 25,540,503, `board_check_info_calls` 2,245,089
(0.295/node), `board_compute_pinned_calls` 2,079,992 (0.274/node),
`board_calculate_checkers_calls` 1,155,770, `board_see_full_calls` 445,100.
Units were reconciled before differencing: generator calls 2,243,478 minus
`compute_pinned` 2,079,992 leaves **163,486**, exactly the `generate_captures`
early-out that increments its counter and returns before computing pins. No
ETW re-profile was requested — it needs an elevated prompt, is a maintainer
job, and cannot make un-shareable work shareable; the post-`5c439da` share
update is derived arithmetically and moves every unchanged region by at most
**0.06 percentage points**, explicitly not a measurement. No implementation,
no games, no timing claim and no Elo claim. Owed to 4.11b.11: a fresh profile
where size decides, and an SEE re-baseline, since the 4.11b.9 board benchmark
showed `threshold SEE only` down 1.77/0.98/1.68% in all three rounds —
consistent-signed, most likely code layout after `make_move_inner` grew
468 -> 542 instructions. Reopen only for a genuinely new consumer of pin or
check geometry, never on donor-engine similarity. At this closure point,
4.11b.11 is next. Evidence: `analysis/pin_check_sharing_2026-09-08.md` and
`tools/results/pinshare-411b10/` (ignored, local).

## RAR-M35 (Measurement, harness and tuning)

**RAR-M35 -- 4.11b.11 incremental SEE attacker maintenance, COMPLETE 2026-09-08;
`NO_CHANGE`, production path withdrawn.** Baseline `8d7da2c` on `dev`; contract
frozen in `tools/results/see-kernel-411b11/registration.md` before any timing,
with correctness gates run first and no throughput number observed at freeze.
The candidate carried an all-colour attacker set (`attackers_to(target, occ) &
occ`) built once per exchange, selected via `attackers & color_occ(side)` --
exactly reproducing `attackers_to_color`, since each colour-specific term of
`attackers_to` is a subset of that colour's occupancy -- and extended by
`see_expose`, which adds only the ray vacating the source can open (diagonal for
pawn/bishop/queen, orthogonal for rook/queen; a knight attacking the target is
never aligned with it, and a king recapture ends the exchange first). All
4.11b.5 semantics were preserved, including the per-candidate selected-king
legality test, the `& !Bitboard::from(target)` exclusion, promotion/new-victim
values, threshold parity, and retention of an illegal candidate in the carried
set. **Correctness was exact and verified beyond the fingerprint**: `bench 13`
7,601,220 / EBF 2.474; all 41 external fixtures (`see_contract` 8/8),
`see_pins` 6/6, debug 275/275, release 276/276, fmt and Clippy `--all-features
--all-targets` clean; plus a `debug_assert_eq!` comparing the carried set with a
fresh `attackers_to_color` on EVERY SEE call, **proven live** by deliberately
dropping queen orthogonal exposure, which made
`threshold_parity_on_deterministic_legal_walks` panic immediately. Rejection is
on throughput. The registered two-stage design required stage 1 -- three
alternating `board_v2` rounds -- to improve `threshold SEE only` in all three
rounds with a median of at least +5%. Measured **-2.92% / -10.42% / -0.69%**,
median -2.92%, zero rounds up, host ambient 5.38%. **Stage 2 was therefore never
run**, saving the entire expensive full-search arm. Round 1 was disturbed on
unrelated columns (`make/unmake only` -4.79%), so the honest effect is rounds 0
and 2: a **1-3% regression**; no round was discarded. Calibration: the
registration named this exact failure mode before exposure -- short exchanges
gain nothing because the initial `attackers_to` builds both colours where the
old first step built one, and `see_ge_impl` exits early on much of its 7.55M
threshold calls -- so direction was a HIT while the predicted +5% to +20% upside
band was a MISS. **A leaf premise is corrected**: the two `attackers_to_color`
calls per exchange step are not duplicates. The second is the mandatory
per-candidate king-legality test at a different square under a different
occupancy, which a carried target-attacker set cannot serve; future SEE work
must target that test, not the attacker set. `src/` was restored byte-identical
to `8d7da2c` with the fingerprint re-verified after withdrawal. No games, no
Elo, no timing claim retained. A fresh ETW profile is still owed and is now more
valuable, since it can attribute SEE's 5.3% between recapturer rebuild and
legality test; it needs an elevated prompt and is a maintainer job. At this
closure point, 4.11b.12 is next. Evidence:
`analysis/see_kernel_2026-09-08.md` and `tools/results/see-kernel-411b11/`
(ignored, local).

## RAR-M36 (Measurement, harness and tuning)

**RAR-M36 — full-search board profile refreshed at head, COMPLETE 2026-09-08.**
Source `2d621ff`; production `a3cca8dc...`, PDB `c61e93e3...`, 162,846 process
samples, five cohorts, 600,000 nodes, 5 repeats. **Recipe recovered.** RAR-M30's
per-sample attribution was a side effect of xperf failing to discover the PDB;
`952711f` fixed that discovery and silently switched the report to per-function
aggregation, where board work inlined into `negamax`/`evaluate` is charged to
those functions and `summarize_board_search_etw.py` — reading a fixed column
that had been correct for the per-address table — resolved `limit`, the byte one
past each function's end, while reporting "100% of engine samples resolved". The
working recipe is to deny xperf symbols on purpose: empty `_NT_SYMBOL_PATH`,
empty `_NT_SYMCACHE_PATH`, and no `rarog.pdb` beside the executable; an empty
symbol path alone is insufficient because xperf reuses its symcache and dbghelp
finds an adjacent PDB first. The PDB must then be restored beside the executable
for llvm-symbolizer, which resolves by the embedded name rather than `--pdb`.
Both directions are now detected from the DATA (`base == limit, size == 0` is
per-address and accepted; non-zero size is per-function and refused with the
regeneration recipe). Refreshed shares against RAR-M30: generation/legality
**6.556%** (6.751%), make/unmake **6.677%** (7.143%), SEE **5.239%** (5.304%),
check queries **5.179%** (5.177%); mechanisms piece relocation **2.752%**
(2.998%), gives_check **1.654%**, check_info **1.026%** (0.912%), compute_pinned
**0.979%** (1.003%), king square lookup **0.502%** (0.544%). **The instrument
validates independently**: RAR-M33's +0.876% whole-search from an ~18% local
make/unmake gain requires that region to be ~6.3%, and this profile reads 6.677%
against RAR-M30's 7.143% — the drop is 4.11b.9, measured by a second instrument
that knew nothing about it, while check_queries reproduces to within 0.002pp. A
**stale mechanism marker** was found and fixed: `piece_relocation_helpers` keyed
only on `::remove_piece`/`::add_piece` and under-read at 1.419% after 4.11b.9
fused the QUIET path into `Board::move_piece`; with `::move_piece` added it
reads 2.752%. The symbolized per-function view additionally shows
`see_recapturer` at **4.35%** exclusive against `see_ge_impl` at **0.87%** —
independent measured support for RAR-M35's conclusion that SEE cost sits in the
per-candidate king-legality test, not the attacker set. It cannot split the two
`attackers_to_color` calls inside `see_recapturer`; that needs a counter or an
`#[inline(never)]` probe. No engine change, no games, no Elo claim. Evidence:
`analysis/board_search_profile_2026-09-08.md`.

## RAR-M37 (Measurement, harness and tuning)

**RAR-M37 — 4.11b.12 king-square caching, COMPLETE 2026-09-08; research
disposition `NO_CHANGE`, no prototype built.** Source `edfb35b` on `dev`. The
leaf's conditional trigger — material cost remaining after shared-geometry work
— is not met: RAR-M36 reads king-square lookup at **0.502%** (RAR-M30 0.544%),
and 4.11b.10/4.11b.11 both closed without touching the board, so the small move
is only 4.11b.9 re-weighting shares. **The register asked for a predeclared
practical floor and none was registered**; the measurement is already exposed
twice, so declaring one now would be choosing a number to fit a result, and the
decision deliberately does not use one. It rests on instrument capability: the
2x-local whole-search ceiling is **0.25%**, against RAR-M33's measured bootstrap
half-width of **1.003%** and RAR-M35's projected 0.6-0.7%. The best possible
version is two to four times smaller than the uncertainty of the gate that must
accept it, so it cannot reach `LOCAL_QUALIFIED` however written. The realistic
gain is smaller than the ceiling — `king_sq` is one bitboard load plus a
`tzcnt`, a cache swaps the `tzcnt` for a field load and removes no memory
access, and 0.502% is an overlapping share already counted inside the
generation, check-query and SEE regions. Cost side: maintenance through castling
but not promotion or null moves, restoration on undo for both colours, copying
on worker cloning, and new fields in independent consistency reconstruction —
the class of derived state whose staleness went undetected in 4.11b.5. Retry
trigger: king lookup above **2%** in a profile AND a caller that invokes it in a
loop rather than once per node; donor similarity is explicitly not a trigger. No
engine change, no prototype, no games, no Elo claim. At this closure point,
4.11b.13 is next. Evidence: `analysis/king_square_cache_2026-09-08.md`.

## RAR-M38 (Measurement, harness and tuning)

**RAR-M38 — 4.11b.13 history capacity and mutation contracts, COMPLETE
2026-09-08; tightened and integrated in `f70ac19`.** Baseline `745976b` on
`dev`. Behaviour-neutral, **no speed claim in either direction**, per the
register's own condition that zero observed growth events cannot support one.
`Board::reserve_history` (`pub(crate)`) reserves further make/unmake pairs, and
`search_impl` reserves **`MAX_PLY`** on the root once before any hot path or
helper exists; `Board::clone` preserves capacity, so every worker's
`root.clone()` inherits the reservation and no thread reallocates while
searching. The history stays a `Vec`: a clamped fixed array would silently drop
repetition evidence in a long game. **The gap was real but invisible to the
instrument that looked for it** — peak depth is `game_plies + search_depth`, so
an ordinary 64-move game hits `len == capacity` at 128 and reallocates on the
next search's first push, yet RAR-M30 measured zero growth across 25,718,154
pushes because `bench` builds every position from FEN and leaves game history
empty. `is_legal` audited: `Move::from_uci` always yields `QUIET`, so only
`legal_move`'s canonical move carries real flags and a caller playing its own
input corrupts make/unmake; `is_legal` has **no production callers** (three test
assertions that never play the move) and all five search sites bind what
`legal_move` returns, so the property the leaf asked to preserve holds. It is
now documented and pinned by a test asserting raw flags are `QUIET` while
canonical flags differ, across a double push, a capture and a castle. Five
contract tests cover headroom, no reallocation across a 128-ply walk, clone
inheritance plus the clone walking without reallocating, exact hash/history
restoration on a 64-ply unwind, and canonicalization. **The clone test was
proven live**: reverting `Clone` to `Vec::with_capacity(self.history.len())`
made it fail, and only it failed. Verification: fresh no-feature build
reproduces **7,601,220 / EBF 2.474**; debug **280** / release **281** tests,
fmt and Clippy `--all-features --all-targets` clean. No public surface widened —
`reserve_history` is `pub(crate)` and the tests live in a `#[cfg(test)]` module
inside `board.rs` so the private field stays private. No games, no Elo. At this
closure point, 4.11b.14 is next. Evidence:
`analysis/history_contracts_2026-09-08.md`.

## RAR-M39 (Measurement, harness and tuning)

**RAR-M39 — 4.11b.14 larger board representation, COMPLETE 2026-09-08; research
disposition `NO_CHANGE`, no comparison registered and no implementation
opened.** Source `0d69c5f` on `dev`. The leaf's gate — open an implementation
only if the preceding profile still identifies substantial representation cost —
is **not met**: RAR-M36 puts no board region above **6.7%** (make/unmake 6.677%,
generation and legality 6.556%, SEE 5.239%, check queries 5.179%), and those are
the costs of doing the work rather than of the representation, which a different
layout trades rather than deletes. This session's two direct experiments concur:
4.11b.9 won +0.876%, 4.11b.11 lost 1-3% on its own benchmark. **Six type boards
plus colours, rejected on a measured trade**: `Board` is **264 bytes**, the
variant saves **48** (Rarog already keeps `occupancy[2]` and `all_occ`), both
sit far inside L1 with neither near a meaningful boundary, and against that
every `pieces(color, piece)` gains a load plus an AND across **208 call sites**,
**102 in `eval.rs`**, the profile's largest region at 29.49% exclusive.
**Per-ply state copying, quantitatively worse**: `UnmakeInfo` is **24 bytes** so
a 128-ply stack costs **3 KiB** and stays in L1, whereas copying whole board
state in the shape of Reckless's `InternalState` costs **128 x 264 = 33 KiB**
and leaves it — elevenfold, to avoid inverse work 4.11b.9 reduced to a single
fused mask. **Selectively checked generation, already amortized**: RAR-M34
measured 422,246 staged quiet generations served at zero extra `compute_pinned`,
and `board_gives_check_fast_calls` 25,540,503 against
`board_gives_check_full_calls` 49,385 is about **517:1**, so deferring legality
would trade a shared per-node cost for a per-move cost on a population pruning
discards unexamined. The only change made is a compile-time guard pinning the
two footprints the decision rests on, as upper bounds (`Board <= 264`,
`UnmakeInfo <= 24`) since padding may differ between supported targets and only
growth invalidates the argument; each message names this leaf so a breaking
field addition fails the build. **Proven live** by adding a `[u64; 4]` field,
which failed the build with the intended message. Behaviour unchanged at
**7,601,220 / EBF 2.474**; debug **280** / release **281**, fmt and Clippy
`--all-features --all-targets` clean. Retry trigger: a single board region above
**12%** AND a named mechanism that removes work rather than relocating it; donor
similarity is explicitly not one. No games, no Elo, no NNUE interaction — full
NNUE stacks remain Phase 5. At this closure point, 4.11b.15 is next. Evidence:
`analysis/representation_2026-09-08.md`.

## RAR-M40 (Measurement, harness and tuning)

**RAR-M40 — 4.11b.15 draw-state policy boundary, COMPLETE 2026-09-08; research
disposition `NO_CHANGE` on all four policies.** Source `95db376` on `dev`;
engine source untouched, so `bench 13` holds at **7,601,220 / EBF 2.474**. The
only change is `tests/draw_semantics.rs` (`df94b7d`). **What RAR-S18
establishes and what it does not**: arm A (null-clock + cross-null fence +
root-aware) **−7.21 ± 6.03**, arm B (same without root-aware) **−11.91 ± 7.67**;
both exclude zero so both bundles were harmful, but **neither isolates a single
part**, and although B is worse than A by 4.70 the intervals overlap heavily
(`[−13.24,−1.18]` vs `[−19.58,−4.24]`) so that ordering is unsupported. No
disposition leans on these as evidence about one component. **(1) Rule-50 clock
— KEEP, no retry trigger**: mate on the 100th-clock move outranks the draw, four
tests cover it. **(2) Null-move boundaries — KEEP**: the cross-null question
resolves structurally rather than on Elo, since `is_repetition` compares the
full hash including side to move so crossing a null yields only false NEGATIVES,
and `can_declare_draw` is reached only from `game_result` and the root tablebase
gate where history holds no nulls; the rejected fence guarded a search scoring
imprecision, not a legality defect. Retry needs a measured case where a
cross-null match changes a **root** best move. **(3) Pre-root versus in-search
repetition — KEEP**, and partial root-awareness already exists: `search.rs:2218`
guards `ply > 0`, so the root is never scored a draw in search and the rejected
change was a further one; retry needs a demonstrated **won game** lost to the
aggressive twofold, not node counts. **(4) Repetition versus TT and evaluation
keys — KEEP, audited clean, now pinned**: repetition uses the position hash
only; the TT key is `board.hash` with the clock applied on READ as a mate-score
correction in `tt::score_from_tt`; the eval cache stores a `halfmove_clock`
compared for equality (`eval.rs:1254`) as entry validity, not in a hash. Two new
tests pin this — one asserting positions differing only in the clock share a
hash, one recording that the scan bound is a **cost** choice because an
irreversible move permanently changes the hash. **Proving the identity test live
took three sabotage attempts**: via `check_consistency` (never called by these
tests) and via `from_fen` before the clock is parsed (still zero, a no-op),
before mixing it in after parsing failed the test and only it — a sabotage that
does not visibly change the thing under test proves nothing. Debug **282** /
release **283**, fmt and Clippy `--all-features --all-targets` clean. No games,
no Elo, no playing change proposed and no bundle rescued. At this closure point,
4.11b.16 is next. Evidence: `analysis/draw_policy_2026-09-08.md`.

## RAR-M41 (Measurement, harness and tuning)

**RAR-M41 — 4.11b.16 integrated board cluster qualification, COMPLETE
2026-09-08; QUALIFIED, speed claim banked, no Elo claimed.** Registration frozen
in `120b8d9` before the run; arms `1d720af` against head `1be34ac`, which is
exactly the fused relocation, history reservation and footprint assertions. Both
arms **behaviour-identical**: all six PGO binaries reproduce `bench 13` at
**7,601,220 / EBF 2.474**, so trees match and fixed-node NPS is a clean
throughput comparison. Section entry was deliberately not the baseline, because
the 4.11b.5 SEE repair changed the fingerprint and measuring across it would
confound throughput with a correctness fix owned by 4.11b.17. **Correctness
matrix, all passing**: debug **282** / release **283**; `see_contract` 8/8 with
all 41 external fixtures; `see_pins` 6/6; `draw_semantics` 8/8; randomized
`board_differential` and `fuzz_lite`; **72** tests under the PEXT slider backend
(`--cfg rarog_pext -C target-cpu=native`); fmt and Clippy `--all-features
--all-targets` clean; feature-off default. The one difference from section entry
is the SEE repair, and six independent fingerprint checks confirm every later
board change preserved it. The supported-target cross build was **not runnable
on this host** — `aarch64-pc-windows-msvc` is not installed and the vendored
fathom C code needs a cross `cl.exe` — and is stated as owed to the ARM64
compatibility host rather than claimed. **Instrument**: six PGO binaries via
`cargo xtask build --arch pext --native --pgo`, three per arm, rotating so no
build carries an arm; PGO build variance **verified, not assumed**, two builds
of identical source differing by hash. A **null pair** of same-revision builds
measured **+0.222%, 95% [-0.130%, +0.630%]** — containing zero, so the
instrument is unbiased, and that upper bound became the effective floor over the
0.5% practical floor derived from the ~2 Elo per 1% NPS constant. **Main
comparison**: 96 alternating pairs at 2,000,000 nodes, seed 4119, one discarded
warm-up per binary, 1T, Hash 16 MiB, rustc 1.97.1 PEXT `target-cpu=native`, no
affinity pinning; **+1.421%, 95% [+0.953%, +1.764%]**, median 3,606,933 ->
3,658,176 nps, **91/96** pairs faster, max host busy **9.11%** against a 15%
gate. Every registered condition passed and the frozen rule banks the claim.
**Calibration**: projected ~0.5% half-width from RAR-M33's measured 1.003%, got
**0.405%** — deriving the projection from a measured width rather than a
variance model corrected RAR-M33's miss; the estimate exceeds RAR-M33's non-PGO
+0.876% but is consistent with it since that interval contains 1.421%, with PGO
amplification and the two extra commits as unestablished candidate reasons; and
the registration's pre-stated risk that variance might exceed the effect did not
materialise. **Claimed**: +1.421% [+0.953%, +1.764%] whole-search NPS under
production pooled-PGO settings on a verified-idle host with identical behaviour.
**Not claimed**: any Elo — the constant would suggest ~+2.8 Elo but that is an
inference, not a measurement, and no games were played. At this closure point,
4.11b.17 is next and owns the playing gate. Evidence:
`analysis/cluster_qualification_2026-09-08.md` and
`tools/results/cluster-411b16/` (ignored, local).

## RAR-M12 (Measurement, harness and tuning)

### Phase-4 registration (RAR-M12, 2026-08-12)

Registered at 4.0, before any Phase-4 code moves. Caps are prospective, derived
from each cluster's PLAN §4 prior through RAR-M10's drift fit; a cap is a stop
point, never a target to run to, and none of it may be revised after games are
seen.

| Step | Cluster | Prior (nElo) | Bounds | Cap (games) |
|---|---|---:|---|---:|
| 4.5 | A — ordering, histories, LMR | 15–45 | `[3,10]` | 6,000 |
| 4.6 | B — static eval, TT, qsearch | 5–25 | `[3,10]` | 9,200 |
| 4.7 | C — main selectivity | 25–60 | `[3,10]` | 4,000 |
| 4.8 | D — extensions, depth authority | 5–25 | `[3,10]` | 9,200 |
| 4.9 | E — root search and clock | 5–20 | `[3,10]` | 9,200 |
| 4.13–4.16 | F–I — HCE structural | 15–50 each | `[3,10]` | 4,000 each |

That is roughly **55,000 STC games** of cluster gating, before ablations and
the 4.10 / 4.18 / 4.19 checkpoints. RAR-M11's completed schedule is the only
throughput anchor on this host — 320,000 games in about 79 hours, so ~4,050
games/hour — which puts the gating at roughly **14 hours**, call it 20 with
checkpoints. Treat that as an order of magnitude: it came from tune binaries
under an SPSA driver, not from final-PGO SPRT pairs.

Stop rules, all pre-registered:

1. Two fully implemented **search** clusters failing to produce an accepted
   gain stops the track and returns to 4.2–4.3. Not a third attempt.
2. Two coherent **HCE** clusters failing closes track H; go to 4.19 or Phase 5.
3. 4.10 is a real expected-value review with a close option, not a formality.
4. A cluster ends accepted or reverted. Borderline results are not carried.
5. **2.4.0** needs cumulative ≥ +40 Elo STC over 2.3.2 with the 95% lower bound
   above +25, plus positive LTC and 4T lower bounds. The programme *target* is
   ≥ +100 cumulative; a result there with a lower bound above +75 may justify a
   higher minor version.
6. HCE-changing A/Bs and every cross-engine cohort run with adjudication off.

## RAR-S57 (Search and selectivity)

**Amendment to RAR-S57, 2026-08-18.** The bundle passed, but it is **not what shipped.** RAR-S58's ablation showed 4.7c reproduces the whole +24.50 nElo on its own (+24.90 ± 16.01, H1 in 1,810 games), leaving 4.7a at −0.40 nElo marginal, so 4.7a was reverted and the accepted head is the `47c-only` arm — byte-identical in engine source to the binary that passed that SPRT, fingerprint **6,922,439 / EBF 2.451**. Two consequences worth stating plainly. First, the shipped head was gated at `[0,10]`, a weaker bar than the bundle's `[3,10]`; its lower bound is +8.89 nElo and the bundle cleared `[3,10]` at the same point estimate, so the risk is small, but an ablation arm was used as a shipping decision and that is the exception, not the pattern. Second, RAR-S57's headline Elo of +15.44 ± 8.06 describes a configuration that no longer exists; quote **+15.56 ± 10.02** for the shipped one.

## RAR-S70 (Search and selectivity)

**RAR-S70 result note, 2026-08-21 — the answer harness now has an exchange
rate, and it is small.**

The gain is real and it is modest: **+2.33 Elo** (95% CI [0.48, 4.18]),
**+3.58 nElo** (CI [0.73, 6.43]). Two things about reading it.

**First, an SPRT is a decision procedure, not a measurement.** It stopped at
56,928 games because that is when the evidence first separated "at least 3"
from "at most 0" at the registered error rates. Accepting H1 says the change is
real and positive; it never said the gain is large. And the point estimate at
an H1 boundary is if anything biased **upward** — conditioning on having
stopped for H1 selects favourable realisations — so +3.58 nElo is not an
understated reading of a bigger truth. It may be an overstated reading of a
smaller one.

**RAR-M10 predicted this exactly.** `8.3e-6 x 3 x (3.58 - 1.5) x 56,928 =
2.948` against the 2.95 the harness reported. The model has now called the
observed LLR to three decimals on every gate it has been applied to, and should
be used to size every future one before registration.

**Second, and this is the finding that matters: the proxy is now calibrated.**
Oracle agreement moved **66% -> 78%** at depth 12, monotone across three
depths, and it bought **2.33 Elo**. That is roughly **0.2 Elo per point of
agreement**. Extrapolating the remaining gap linearly — and it is only an
extrapolation — closing 78% all the way to 100% is worth **single-digit Elo**.

**So oracle agreement is not where the ~196 Elo deficit lives, and the reason
is structural.** `answer_compare.py` compares the two searches **at fixed
depth**, which factors out precisely the axis on which Rarog and the oracle
differ most: how many nodes it costs to reach that depth. Rarog runs 1.60x the
oracle's quiescence per node and builds a larger tree throughout. At equal
TIME the oracle simply gets more depth, and no fixed-depth agreement metric can
see that.

**Consequence for 4.6c.** The cluster's other member, `LmrMinReducedDepth`,
scores WORSE on the proxy and removes **18.4% of the tree at flat agreement**
— so it attacks the axis the harness is blind to, and its weak proxy showing is
now weak evidence against it rather than strong. It should be gated next, and
the answer harness should not be used to rank candidates that differ in tree
size. Agreement ranks move choice at equal depth; it does not rank strength.

## RAR-S69 (Search and selectivity)

**RAR-S69 stop note, 2026-08-20.** Stopped at 4,642 games: Elo **+4.64 ± 6.49**,
nElo **+7.14 ± 9.99**, LLR **+0.65**. Not promoted. The margin shift stays
unmerged; the branch `p410-margin-relief` carries it and the recipe is the ten
values in the row above.

Stopped on a budget judgement, not on the reading: even at 80,000 this line was
worth a few Elo at best against a ~196 Elo deficit, and the machine is better
spent on 4.6. **The blind-shift line is closed** — RAR-S54's headroom was
re-tested in two halves (LMR at RAR-S68, margins here) and neither half
produced a resolvable gain on the current head.

**Line summary, so nobody re-opens it.** Five consecutive candidates —
killer clearing, the `improving` fallback, 1T jitter, LMR relief, margin relief
— went to a gate and none cleared it, at a cost of ~39,500 games. Three were
design differences from Stockfish; two were blind scalar shifts. Against them,
the one change that DID pay in this phase was 4.7c: a structural contract
replacement ("can this capture bridge the margin" for "does it lose material")
that closed a measured 5.17× divergence against the oracle. **A difference from
Stockfish is not latent Elo, and neither is a scalar.** Candidate selection
should follow measured contract divergence, which is what 4.6 has in
abundance.

## RAR-S68 (Search and selectivity)

**RAR-S68 stop note, 2026-08-20.** Stopped at 4,716 games: Elo **−1.40 ± 6.24**,
nElo **−2.22 ± 9.92**, LLR **−0.44**. Dead flat rather than negative — unlike
RAR-S67, which was clearly the wrong direction by this point. Not promoted; the
`LmrRelief` switch stays at default 0 with this row as its owner.

**Two things this settles, cheaply.** First, my 60,000 cap could only have
decided a truth of ≥ +4 nElo: at `[0,3]`, a true +3 needs 78,715 games and a
true 0 needs the same to reach H0. Running it out would most likely have bought
an unresolved stop, which is why it was stopped at 4,716.

Second, and more useful: **I tested the wrong tenth of RAR-S54.** That probe
shifted **twelve** constants, and only **two** were the LMR table
(`LmrTableBase`, `LmrTableDiv`); the other **ten were pruning margins** —
futility, razoring, LMP, quiet-history and SEE. `LmrRelief` is the LMR half
alone, and it reads zero. So RAR-S54's +4.06 most plausibly lives in the
margins, not in the reduction — and 4.7's +15.56 came from the ProbCut move
filter, a different mechanism entirely, so those ten constants are **untouched
since RAR-S54 measured them**.

That narrows the hypothesis rather than refuting it, which is worth more than
the eight hours confirming "unresolved" would have cost.

## RAR-S67 (Search and selectivity)

**RAR-S67 stop note, 2026-08-20.** Stopped at 3,764 games: Elo **−5.35 ± 7.28**,
nElo **−8.17 ± 11.10**, LLR **−0.90**, tracking to H0 at ~12,900 games. Not
promoted. The switch is retained at default 0 with this row as its owner.

**The candidate was mis-derived, and that is the finding worth keeping.** I
read RAR-S54, RAR-S62 and RAR-S64 as "noise in the selectivity surface gains
Elo" and built symmetric jitter. Re-examining what those three actually changed:

| | change | direction |
|---|---|---|
| RAR-S54 | blind uniform 15% shift, +23.2% nodes | **less** selective |
| RAR-S64 | stale prior-reduction | **less** selective |
| RAR-S62 | ProbCut desync | *more* selective |

Two of three were **directional de-selectivity**. Symmetric jitter has **zero
mean effect** on the reduction — it adds variance without moving the average —
so it cannot reproduce a directional effect by construction. I turned "the bug
read the wrong value" into "noise helps" without checking what the wrong values
did to the search, which is the same reasoning error as the ProbCut episode.

RAR-S62 still points the other way and is not explained by this. Recorded as a
correction to the hypothesis, not as a new one: the supported claim is
narrower, that **reducing less** gains Elo here, which is what RAR-S54 measured
directly and what 4.7 delivered +15.56 of structurally.

## RAR-S61 (Search and selectivity)

**The bracket changed, and RAR-S61 is why.** That gate spent its full 16,000 games at `[3,10]` and returned LLR 0.39, because the candidate landed at 6.92 nElo — 0.42 from the bracket midpoint, the one value it cannot decide. RAR-M10 predicted that drift to three decimals after the fact; used prospectively now, it says `[3,10]` needs **101,205 games** for a true 7 while `[0,10]` needs 17,711. Moving the midpoint from 6.5 to 5.0 moves the blind spot off the value this candidate has already been measured at. `[0,5]` was rejected — 141,687 games for a true 3. ⚠ The fix is not assumed to help: it changes the tree by −2.0% and its sign is unknown, which is precisely why RAR-S61's +4.50 ± 3.50 cannot simply be carried forward. Findings 2 (`improving` has no `ply-4` fallback, 9.7% of nodes affected) and 3 (killers never cleared for descendant plies) are deliberately excluded so this measures the fix and not two untested behaviour changes alongside it. | `tools/test_engines/rarog-45{base,fixed}-pext-pgo.exe`; `analysis/code_audit_2026_08_19.md`; RAR-S61; RAR-S63; RAR-M10 |

## RAR-S54 (Search and selectivity)

**RAR-S54 reconstruction recipe, recovered 2026-08-18.** The probe's source
commit `7693010d` was found DANGLING — its branch `probe/10.0c-less-pruning`
no longer existed and the next `git gc` would have deleted it. The recipe is
recorded here so the experiment is reproducible from this document alone, and
no branch or tag is needed to keep it.

Baseline arm: `c907c2e8` on the then-`development`, bench **5,173,540**.
Probe arm: the 12 values below, bench **6,373,363**. Both final-PGO,
rustc 1.97.1, clean manifests, `git_dirty = False`. Rebuild both, confirm the
two bench fingerprints, and the arms are reproduced exactly — the fingerprints
are what verify the reconstruction, so do not skip them.

"Uniform 15% toward less pruning" means ×1.15 on every margin/threshold that
*permits* a prune, and the reciprocal on the two LMR table constants, so the
reduction gets smaller rather than larger:

| Parameter | Accepted (2.3.1 line) | Probe | Operation |
|---|---:|---:|---|
| `FutilityBase` | 60 | 69 | ×1.15 |
| `FutilityNotImproving` | 42 | 48 | ×1.15 |
| `RazoringCoeff` | 193 | 222 | ×1.15 |
| `LmpBase` | 88 | 101 | ×1.15 |
| `LmpNotImproving` | 63 | 72 | ×1.15 |
| `QuietHistPruneCoeff` | 5,069 | 5,829 | ×1.15 |
| `SeePruningCoeff` | 51 | 59 | ×1.15 |
| `SeePruningMax` | 869 | 999 | ×1.15 |
| `FpBase` | 184 | 212 | ×1.15 |
| `FpCoeff` | 117 | 135 | ×1.15 |
| `LmrTableBase` | 646 | 549 | ×0.85 |
| `LmrTableDiv` | 2,335 | 2,747 | ÷0.85 |

⚠ The "accepted" column is the **2.3.1-era** surface, not today's. `FpBase` and
`FpCoeff` in particular have since moved to 211 and 135 on `dev`. Re-running
this probe against the current head means recomputing the ×1.15 from the
current values, which makes it a different experiment — say so if you do it.

This is what a ledger row has to contain before its branch can be deleted: not
a pointer to code, but the recipe and a fingerprint that proves the rebuild
matched. RAR-S52's and RAR-S53's citations already meet that bar a different
way — every tool they name (`tools/diag_search_quality.ps1`, `src/diag.rs`,
`tools/pgn_depth_at_nodes.py`, `tools/sprt.ps1 -Nodes`) lives on `dev` today.

## RAR-S55 (Search and selectivity)

**Correction to RAR-S55, 2026-08-12.** The originally reported `lmp_prune`
divergence of **13.35x was an artifact** and is withdrawn. Rarog's `lmp_prune`
counts every quiet skipped (per MOVE); the oracle's fired once per node whose
quiets were suppressed (per NODE). The ratio compared moves against nodes — the
RAR-S25 denominator error, this time inside the Phase-4 instrumentation itself.

A per-node `lmp_nodes` was added to both engines and the suite re-run
(`analysis/phase4_differential_v2_depth8.txt`). The corrected reading **inverts
the finding**: Rarog suppresses quiets at **8.3%** of interior nodes against the
reference's **14.6%**, normalised **0.57x**. Rarog fires move-count pruning
*less* often, not an order of magnitude more.

Consequences: the 4.3 mechanism map's second-ranked lead for cluster 4.7 is
withdrawn, and no code was written against it — the artifact was caught while
designing the candidate, one step before it would have justified a change and a
game budget. `lmp_prune` is retained as a Rarog-only volume reading. Everything
else in RAR-S55 stands; its invariants passed then and pass now.

**Second correction to RAR-S55, 2026-08-14.** The reported ProbCut divergence —
`probcut_attempt` **2.33x** and conversion **22.7% against 91.2%** — is
**withdrawn as not comparable**, the same denominator class as the `lmp_prune`
correction above, one lead later.

The oracle's `probcut_attempt` fires inside its MovePicker loop, once per move
actually searched, up to `2 + 2·cutNode` at a node. Rarog's fired once per NODE
entering the ProbCut block, before capture generation, so nodes with no
eligible capture counted an attempt that could never convert. The `2.33x`
compared nodes against moves; the conversion pair divided cuts-per-node by
cuts-per-move-tried.

Compounding it, the oracle's **TT-served shortcut** — a stored entry already
above `probcutBeta`, so the node returns without searching anything — was made
to count as `probcut_attempt` at 4.1 specifically so `probcut_cut` could not
exceed it. That satisfied the invariant and concealed the population. Rarog has
no TT-served ProbCut path at all.

Both engines now carry `probcut_nodes` (per node), `probcut_attempt` (per
move), and the oracle carries `probcut_tt_served`. The runner's invariant
becomes `probcut_cut <= probcut_nodes`; the old form held on both engines while
being incapable of falsifying anything.

**The corrected reading, `analysis/phase4_differential_v3_depth8.txt`.** Same
suite, same depth, same oracle revision; all invariants pass on both engines and
the node ratio reproduces v2's 1.861 exactly. `probcut_nodes` reads 8,237 on
Rarog, the identical value the mislabelled `probcut_attempt` carried in v2, so
the rename is a pure rename and nothing else moved.

| | Rarog | oracle | note |
|---|---|---|---|
| entries per node | 3.019% | 2.046% | norm 1.48 |
| — of which TT-served | 0 | 1,305 | Rarog has no such path |
| reaches the SEARCH stage, per node | 3.019% | 1.156% | **2.61x** |
| moves searched, per node | 2.099% | 0.406% | **5.17x** |
| moves per search-stage entry | 0.695 | 0.351 | 1.98x |
| conversion per search-stage node | **22.7%** | **25.2%** | the gap is gone |
| conversion per move searched | **32.6%** | **71.9%** | 2.2x, not 4x |
| search-produced cuts per node | 0.685% | 0.292% | Rarog **2.35x** the oracle |

**The finding is not withdrawn, it is reshaped, and it moved one level down.**
Three things the old numbers had wrong:

1. **Per node, the conversion gap does not exist.** 22.7% against 25.2%. The
   headline "22.7% against 91.2%" that ranked ProbCut second in the 4.3 map is
   dead.
2. **75.3% of the oracle's ProbCut cutoffs are free.** 1,305 of 1,733 come
   straight from the TT with no search at all. That is what the 91.2% was
   mostly measuring.
3. **Rarog's ProbCut is more productive per node, not less** — 2.35x the
   oracle's search-produced cutoffs per node searched. It is the *price* that
   diverges, not the yield.

What survives is a **move-filter** contract, and it has the same failure shape
as 4.7a one level down: Rarog searches **5.17x** the normalised ProbCut moves
and converts **32.6%** of them against **71.9%**. Two thirds of Rarog's ProbCut
move-searches produce nothing. The mechanism is visible in source and needs no
counter: the oracle admits a capture only when SEE bridges `probcutBeta −
staticEval` and stops at `2 + 2·cutNode` moves, where Rarog admits any
`see_ge(mv, 0)` and tries up to 8.

A second, **separable** finding falls out of the same reading: Rarog has no
TT-served ProbCut shortcut, and the oracle takes one at 0.89% of its nodes for
zero search cost. That is a different contract from the move filter and must
not be bundled with it silently — it is cheap, it is not selectivity, and it
would have to be attributed on its own.

Consequence: 4.7c has a subject again, but **not the one the 4.3 map named**.
It is the ProbCut move filter, not the entry gate, and the TT shortcut is a
separate question. Also note what this says about 4.1's defect list: the
ProbCut issue was *seen* at 4.1 and closed by forcing the invariant to hold.
Making a cross-check pass is not the same as making two counters comparable —
that hid the 1,305 free cutoffs for two phases.

Frozen local Stage-1 package SHA-256 hashes: executable
`DA78A1455BAFE222BD6AF7EF243B8C62450B6BC0913C4AF3B09F8C68E14826E8`;
`rarog_hce.dll`
`E43B602B994A3A2EB86173675C5687E53415911DB63FA6FC5B1F74DB40A3F6D5`.

## RAR-E13 (Evaluation and data experiments)

### RAR-E13 — is the fitted king-safety table worth its tree cost? (registered 2026-09-03, before games)

- **WITHDRAWN 2026-09-03, unresolved, before the boundary.** Stopped at roughly
  a few thousand games with arm B leading by **about +7 Elo**. That number is
  **an observation, not evidence**: AGENTS.md's rule that an unresolved stop is
  not "probably fine" applies exactly here, and RAR-S61 is the standing example
  -- +4.50 +/- 3.50 at LOS 99.41% that turned out to be a stale-read bug.
- **Why it was withdrawn rather than finished.** Arm B could not be adopted
  whatever it measured. The 1,218 slots were fitted JOINTLY, so other terms
  carry compensations for the inflated `king_safety_table`; reverting that one
  block leaves those compensations uncorrected and yields a vector that is the
  optimum of neither the free fit nor a constrained one. PROCESS rule 5 already
  requires inspecting post-fit compensation for materially moved families. The
  same objection is why a subset of a converged SPSA vector is invalid.
  Adopting B would also have seeded every 4.10 refit cycle from a hand-edited
  point.
- **What the +7 is a hypothesis ABOUT, for 4.11.** It is not a speed effect --
  both binaries are the same code at the same nodes/second. It is tree size:
  arm A needs 8,044,078 nodes to reach depth 13 against arm B's 6,972,274, so
  arm A searches shallower at a fixed clock. **These two arms cannot separate
  "worse evaluation" from "evaluation mistuned against margins calibrated for
  the old scale"**, because in arm B those are the same change. 4.11
  recalibrates the eval-coupled margins -- reverse futility, razoring, ProbCut,
  null-move scaling, SEE pruning, aspiration -- and however much of arm A's
  penalty disappears was miscalibration. If most of it goes, arm A with retuned
  margins should beat both. If little goes, the fitted table does not earn its
  cost and 4.10's next cycle should CONSTRAIN it in the fit rather than have
  anyone revert it by hand.

## RAR-E12 (Evaluation and data experiments)

- **Question.** RAR-E12's candidate grew `bench 13` by 12.3% and broke the
  KBN-K endgame floor. An ablation identified **one block as the whole cause**:
  reverting `king_safety_table` to its previously accepted values takes bench
  from 8,044,078 to **6,972,274** -- *below* the current head's 7,165,683 --
  and lifts KBN-K conversion from 0.8980 to **0.9490, above its 0.9184 floor**,
  demoting the dtz breach from blocking (-4.4 SE) to report tier (-2.2 SE).
  So: is the fitted table's middlegame value worth the search efficiency and
  endgame precision it costs?
- **Why the fit inflated it.** `hce-v3` carries **367,664 natural mates against
  `hce-v2`'s 6,428**, a 57x increase, because dropping adjudication stopped
  resigning games out. Most are king hunts, so the corpus contains far more
  evidence that attacking the king pays. **Texel's objective cannot see that a
  more volatile evaluation costs 12% of the search tree** -- it prices label
  agreement, not nodes-to-depth. This is precisely the blind spot PROCESS rule
  10 anticipates when it keeps search parameters fixed during fitting.
- **Arms.** Identical vectors except one block. Arm A is RAR-E12's candidate,
  `rarog-e09cand-pext-pgo.exe`, bench 8,044,078 / 2.481. Arm B is
  `rarog-e09noks-pext-pgo.exe`, bench **6,972,274 / 2.466**, built from
  `d306e21` plus `analysis/artifacts/rar-e13-candidate-eval.patch` (SHA-256
  `7ADC9C44...`) -- the same fit with `king_safety_table` alone restored to the
  accepted head's 40 values. Both binaries were built from a dirty tree, so
  both patches are committed; a rebuild must reproduce those fingerprints.
- **Registered gate.** Arm B versus arm A, **`[-5,5]` nElo**, alpha = beta =
  0.05, cap **30,000 games**, `3+0.03`, Threads 1, Hash 64 MB, concurrency 14,
  paired UHO random order, no adjudication. **The bracket is symmetric because
  the sign is genuinely unknown** -- the table is a fitted quantity and may
  carry real middlegame value that outweighs its tree cost. AGENTS.md's
  asymmetric `[0,3]` would presume B must earn the change; here neither arm is
  the status quo, since arm A is itself unadopted. RAR-S62 resolved a symmetric
  `[-5,5]` in 4,436 games.
- **Stop/disposition.** B wins or ties within the bracket -> **adopt B**, which
  banks RAR-E12's evaluation gain with a smaller tree than the current head and
  a passing conversion floor. A wins -> the inflated table earns its cost, and
  adopting A then requires the recorded waiver RAR-E12 called for, with KBN-K
  assigned an owner and a retry trigger. Either way the **+11.81 Elo of
  RAR-E12 is not re-litigated**; this gate only decides which of two vectors
  carries it.
- **What this gate does NOT settle.** KBN-K occurs in roughly 0.28% of games
  (RAR-E10), so neither this gate nor RAR-E12 can measure that family
  directly. The floors instrument exists because the strength gate cannot see
  rare endgames, and a passing SPRT is not evidence that a floors breach is
  harmless.

### RAR-E12 — hce-v3 complete refit gate (registered 2026-09-03, before games)

- **Question.** Does a complete 1,218-slot refit on the 4.9a.6 corpus beat the
  accepted RAR-E08 head in games? The corpus changed in three ways at once --
  row count (2,300,000 -> 3,500,000), phase mix (balanced book -> phase-weighted
  book), and label provenance (52.2% adjudicated -> 0.007%, 6,428 natural mates
  -> 367,664). **This gate prices the combination and cannot attribute the
  result to any one of them.** That is a deliberate cluster under the strength
  rule, not an oversight, and it is written here so no post-hoc attribution can
  be made later.
- **Baseline.** `tools/test_engines/rarog-e08head-pext-pgo.exe`, git `a52f4d2`,
  clean tree, bench **7,165,683 / 2.462**.
- **Candidate.** `tools/test_engines/rarog-e09cand-pext-pgo.exe`, bench
  **8,044,078 / 2.481**. Built from `d306e21` plus
  `analysis/artifacts/rar-e12-candidate-eval.patch` (SHA-256 `0A7187F8...`),
  which is the diff produced by baking
  `analysis/artifacts/rar-e12-final-vector.txt` (SHA-256 `EA932B46...`) with
  `tools/texel/bake_params.py`. **The binary was built from a dirty tree and is
  therefore not reproducible from a git SHA alone** -- the patch and vector are
  committed here precisely so the recipe does not dangle, which is the RAR-S54
  failure. A rebuild must reproduce 8,044,078 / 2.481.
- **Corpus.** `hce-v3-tb`, manifest SHA-256 `07BD88CD...`, 602,619 independent
  starts, `datagen-v2`, book `phase_book_v1.epd` SHA-256 `31E9B655...`, K pinned
  at 1.36439, frozen test opened exactly once.
- **Registered gate.** Candidate versus baseline, **`[0,3]` nElo**, alpha = beta
  = 0.05, cap **80,000 games**, `3+0.03`, Threads 1, Hash 64 MB, concurrency 14,
  paired UHO random order, **no adjudication**. The bracket is the project
  default and must be passed explicitly: `-Mode gainer` defaults to `[3,10]`,
  which AGENTS.md records would drive a true +4 to H0.
- **Stop/disposition.** H1 does **not** by itself accept the candidate. The
  KBN-K floors breach is a blocking-tier failure under a threshold set
  prospectively, so acceptance additionally requires either a repair that
  restores the floor, or an explicit waiver recorded with its reason. H0, or a
  cap-out, closes the candidate and makes the +12.3% bench the first suspect --
  in which case 4.11's search-parameter remeasurement becomes a prerequisite of
  a refit rather than a follow-up to one.

## RAR-E08 (Evaluation and data experiments)

### RAR-E08 — label-contract gate (registered 2026-09-02, before games)

- **Question.** Should a Texel fit learn the literal self-play result on
  positions the tablebase can adjudicate, or the tablebase's verdict? Texel
  fits the value realizable by the consuming search, which argues for self-play
  labels; against that, self-play labels are self-reinforcing, and RAR-E09
  found the mechanism concretely -- KR-K, a 100% theoretical win, is labelled a
  draw on 75% of its `hce-v2` positions.
- **Arms.** One game set, two label sets, differing in exactly one way.
  Arm A is `hce-v2` and the accepted head's vector. Arm B is
  `hce-v2-tb` -- byte-identical rows, FENs, order and split membership, with
  only <=6-man labels replaced by Syzygy truth and cursed wins counted as
  draws. **30,480 train labels changed, 1.325% of rows.**
- **Arm B fit.** `hce-fit-20260902_094603`, started from the accepted vector
  (source SHA-256 `BAD51F3E...`, verified equal to RAR-E06's final vector).
  Final vector SHA-256
  `6BCD3AB015C410ECDE77E2ABA6BA87C14AAB6A189CD5F5389F29F082C1C18B91`,
  K pinned at 1.3806, frozen test opened once. **350 of 1,218 slots differ from
  arm A.** Candidate fingerprint **7,165,683 / 2.462** against the accepted
  head's 7,226,051 / 2.460.
- **The offline losses are NOT comparable and are recorded only as within-arm
  numbers.** Arm B improved its own frozen test by **0.000181**
  (0.11598764 -> 0.11580664); RAR-E06 improved its own by 0.00078088. The
  targets differ, so a loss measured against different targets is not a
  comparison. That arm B's improvement is the smaller of the two is consistent
  with RAR-E09 -- the accepted vector was already closer to tablebase truth
  than its own labels were -- but it is an observation, not evidence for
  either arm.
- **Registered gate.** Arm B versus arm A, **`[0,3]` nElo**, alpha = beta =
  0.05, cap **80,000 games**, `3+0.03`, Threads 1, Hash 64 MB, concurrency 14,
  paired UHO random order, **no adjudication** (the harness default since
  RAR-M17). The bracket is the project default rather than a symmetric one
  because the decision is asymmetric: self-play labels are the status quo, and
  arm B must earn the switch. A symmetric `[-5,5]` would also have no preferred
  outcome at a true zero and would run to the cap.
- **Stop/disposition.** Only H1 adopts tablebase-corrected labels as the
  contract for 4.9a.6's regeneration and every later fit. H0, or reaching the
  cap without H1, keeps self-play labels -- which is the cheaper status quo and
  a legitimate result, not a failure. No offline loss, LOS or post-hoc interval
  substitutes for the boundary.

#### RAR-E08 verdict — ACCEPTED 2026-09-02

- **Result.** H1 at **13,432 games**: W 3668 / L 3408 / D 6356, 50.97%.
  **Elo +6.73 +/- 3.82, nElo +10.34 +/- 5.88**, LOS 99.97%, DrawRatio 41.60%,
  PairsRatio 1.12, Ptnml(0-2) [273, 1580, 2794, 1752, 317], LLR 2.95 against
  +2.94. Wall time 2h30m. **Zero time forfeits in 13,432 games**, no
  adjudication, manifest complete.
- **Prediction check.** RAR-M10 projected the boundary at ~17,000 games from
  the +8.45 nElo reading at 4,842; it resolved at 13,432 with the estimate
  firming to +10.34. The model was right about the shape and slightly
  conservative about the pace.
- **Artifacts.** `sprt_E08TbLabels_vs_E08SelfPlay_20260902_100039.*`, PGN
  SHA-256 `BCEF730E54A07382C6759D3AFA2826FC2566A17FF5EB94304F3005D3F7401273`,
  log SHA-256
  `E9ACB84627B17CF4D9CF0CE39EA6234B77131BBD7C000BA6C8F7BDE9849A0511`, seed
  260902. Arm B fit `hce-fit-20260902_094603`, final vector SHA-256
  `6BCD3AB015C410ECDE77E2ABA6BA87C14AAB6A189CD5F5389F29F082C1C18B91`.
- **What is adopted, precisely.** The **post-hoc relabel** of positions with 6
  men or fewer, cursed wins counted as draws, applied to an otherwise unchanged
  corpus -- that is `tools/texel/relabel_tb.py`, and it is what won. It is
  **not** `datagen-v3`, which adjudicates the GAME on tablebase truth and
  therefore changes the recorded result of every position sampled from that
  game, including opening and middlegame ones. `datagen-v3` remains untested;
  adopting it because "tablebase labels won" would be adopting a different
  change.
- **What this says about the labels.** Texel fits the value realizable by the
  consuming search, and that principle predicted arm A would win. It lost by
  10.34 nElo. The self-reinforcing loop was the stronger effect: RAR-E09
  measured KR-K, a 100% theoretical win, labelled a draw on 75% of its corpus
  positions, with the evaluator already predicting 0.849 against a label mean
  of 0.625. Correcting 1.325% of rows was worth +6.73 Elo.
- **Conversion cost, resolved at n=400.** The endgame floors failed at n=100 on
  four of 57 comparisons. Re-measuring both binaries on 400 paired positions
  says most of that was sampling: **KBN-K 95.0% -> 95.5%** (+0.5 pp, SE 1.5 --
  the -5.1 pp reading was noise, in the very family 4.9a.4 had just fixed),
  KNN-KP -7.8 pp at 1.4 SE, KP-KP -3.0 pp at 1.25 SE. One is real:
  **KQ-KP 98.5% -> 94.7%**, -3.8 pp at 2.9 SE. That is queen versus pawn, where
  coverage is a narrow fortress partial and occurrence is 1.17% of games, so
  the expected-value cost is about 0.04 against a +6.73 Elo gain. Owner
  **4.9a.14**; retry trigger is the refit at 4.9a.27. Aggregate weighted
  conversion is flat: 83.24% -> 83.45%.
- **SUPERSEDED and corrected by RAR-M24, 2026-09-06.** The preceding v1
  aggregate is invalid. The matched v2 full cohort is **1255/1372 = 0.9147 ->
  1254/1372 = 0.9140**. The matched v2 400-position focus rerun reproduces
  every preceding family result, including **KQ-KP 390/396 -> 375/396, -15,
  -3.79 pp**. The KQ-KP conversion debt therefore survives as historical
  causal evidence, but RAR-M21 establishes that the current 60k shortfall
  closes at 200k and 600k. The old text remains above as history.
- **Which metric caught it.** The n=100 floors flagged KQ-KP on
  **DTZ-progress**, and the n=400 conversion re-measure then confirmed a real
  regression there -- while the conversion flag on KBN-K was a false positive.
  DTZ progress was the leading indicator, which is an argument for keeping it
  in the floors rather than reducing them to conversion rate.
- **Scope limitation, recorded at disposition.** `hce-v2` was generated under
  `datagen-v1`, where 52.2% of games ended by adjudication and 98% of decisive
  results were called by the resign rule. Arm B corrects only <=6-man labels,
  so every position above 6 men still carries a truncated result. This gate
  therefore establishes that TB correction pays **on an adjudicated corpus**;
  4.9a.6's regeneration removes the adjudication defect and the balance may
  differ there.

## RAR-E06 (Evaluation and data experiments)

### RAR-E06 — complete HCE refit gate (registered 2026-09-01)

- **Hypothesis.** Recalibrating the complete existing HCE surface against
  phase-balanced, pure self-play WDL improves playing strength despite the
  measured speed cost. The candidate is one indivisible vector: 439/1,218
  slots differ from source and all current linear/nonlinear families may
  interact through score scale, qsearch and pruning consumers.
- **Offline qualification.** Fit vector SHA-256
  `BAD51F3E0AB56B3283C56EC4E06317AC6F4C21109DFDAEA0B833673E773F657E`.
  Fresh confirmation `hce-confirm-20260831_230548` used 150,000 independent
  pure-WDL games from unique book entries 600,001--750,000 and an untouched
  127,778-position phase-balanced test: source **0.12330291**, exact rounded
  candidate **0.12252203**, delta **-0.00078088**; every registered broad
  cohort improved. Candidate bounds/tests are valid. Candidate fingerprint is
  **7,226,051 / 2.460** versus source **6,977,070 / 2.466**.
- **Baseline / candidate.** Baseline `6357856e21219d040d5bac7cba13e95c3107e4a4`;
  candidate `5188eca576755932b31ad634af7821cae5291cf3`. Only `src/eval.rs`
  differs in engine behavior. Baseline binary
  `rarog-hce-refit-base-pext-pgo.exe`, SHA-256
  `04572BA2AC87C9A8E334D838D98A2E074C87232180DA8DEFAF1BFAFC4E5AC481`;
  candidate `rarog-hce-refit-candidate-pext-pgo.exe`, SHA-256
  `4F0465C53143C5E675E42B631AD21175E2E89E605F639FE6A94D6F678C293664`.
  Both manifests record clean `pext-pgo` builds with rustc 1.97.1
  `(8bab26f4f 2026-07-14)` and exact bench verification.
- **Pooled speed diagnostic.** Three independently profiled binaries per arm,
  10 interleaved cycles, `bench 13 3`: pooled median source **3,000,899**,
  candidate **2,965,141**, delta **-1.19%**, bootstrap 95% CI
  **[-2.29%, -0.48%]**; best-of delta -2.25%. Recipe:
  `tools/nps_multibuild.ps1 -Cycles 10 -Repeats 3` with the three
  `hce-confirm`/`hce-refit-base` binaries and three `hce-refit-candidate`
  binaries. This is diagnostic; the clock gate prices the cost.
- **Calibration disposition.** No new null is required. Basilisk and Rarog use
  the same fastchess binary, paired book, 1T `3+0.03`, concurrency 14 and exact
  physical-core affinity instrument. `-NoAdjudication` only omits draw/resign
  termination symmetrically; it changes game duration and outcome variance,
  but neither engine placement nor colour pairing. The maintainer explicitly
  accepted the shared-harness calibration on 2026-09-01 rather than spending
  another 30,000 identical-engine games. The real gate's anomaly checks remain
  mandatory.
- **Harness repair before launch (2026-09-01).** The registered command could
  not start. `d2c7788` rewrote `sprt.ps1`'s option-advertisement guard and
  dropped its empty-list early return; because `$splitOpts` unrolls an empty
  result to `$null` and `[string[]]$null` rebuilds a one-element array holding
  `$null`, every gate invoked **without** `-OptionsA/-OptionsB` threw
  `does not advertise:` with an empty name. The same `$null` also emitted a
  bare `option.` argument to fastchess on that path from `ce4a334` onward. No
  gate had run since `d2c7788`, so no recorded result is affected; the last
  options-free run (`sprt_SearchCore_vs_Head_20260822_101254`) predates the
  guard rewrite and logged no fastchess option warning. Fixed by returning the
  array with `,@(...)` and restoring the empty-list return. Verified in three
  directions: options-free now starts, a bogus option still aborts by name, and
  a valid option list still reaches the manifest.
- **No-adjudication wire proof.** `-NoAdjudication` had never played a game.
  A 20-game `-Mode fixed` run of the exact gate pair
  (`sprt_SmokeCand_vs_SmokeBase_20260901_070934`, seed 4242) ended **20/20 by a
  rules result** — 12 mates, 6 threefold, 1 fifty-move, 1 insufficient material
  — with zero adjudication terminations in either PGN or log, and the manifest
  recorded `adjudication: none`. The flag is live.
- **Registered gate.** Candidate versus baseline, `[0,3]` nElo, alpha=beta
  0.05, cap **80,000 games**, seed **918274631**, `3+0.03`, Threads 1,
  Hash 64 MB, concurrency 14 on physical CPUs
  `0,2,4,6,8,10,12,14,16,18,20,22,24,26`, paired UHO random order, and
  **no draw or resign adjudication**. Book SHA-256
  `7A7F6470615A69C6CF23D565417701D38732876F480AF90D67B42ABADE35644A`;
  fastchess alpha 1.8.0 SHA-256
  `8444E73965AE44E716CDE1BB546A7D7C8C9FC7A442A44194A0C71A3BFFA7DD0D`.
  RAR-M10 predicts about 47,200 games at true +4 nElo and about 78,700
  at true +3 or 0 under its strength-v2 calibration; applying it to no
  adjudication is explicitly an extrapolation, so the cap is conservative and
  is not extended after games begin.
- **Stop/disposition (as registered).** Only fastchess H1 accepts the entire
  vector. H0, any anomaly, or reaching 80,000 without H1 rejects and restores
  the baseline HCE. No point estimate, LOS, offline loss or post-hoc interval
  substitutes for the registered boundary.

#### RAR-E06 verdict — ACCEPTED 2026-09-01

- **Result.** H1 accepted at **3,914 games**: W 1206 / L 958 / D 1750, points
  2081.0 (53.17%). **Elo +22.04 +/- 7.51**, **nElo +32.05 +/- 10.88**, LOS
  100.00%, DrawRatio 38.68%, PairsRatio 1.38, Ptnml(0-2)
  [84, 421, 757, 553, 142]. LLR 2.95 against the (-2.94, 2.94) boundary.
  Wall time 44 minutes. It resolved far short of RAR-M10's 47,200-game
  estimate because that estimate was for a true +4 nElo and the measured
  effect is +32.
- **Artifacts.** `tools/results/sprt_HCERefit_vs_HCEBase_20260901_072106.*`.
  PGN SHA-256
  `A1B621C7CED422BA130EBEC229A4916FA361BEE20DE990AA10B4DBC265CFBA34`; log
  SHA-256
  `B1BD467E6566A198D73C7FDC3458AFECC7209494D3CA76CD72A872689988B71D`. **Both
  hashes were computed after the run, not by the runner**: the anomaly guard
  threw before `sprt.ps1` appends its completion lines, so the manifest
  carries `started_utc` but no `completed_utc`/`pgn_sha256`/`log_sha256`.
- **Anomaly and its disposition.** The match tripped `Assert-NoMatchAnomaly`
  on 3 time forfeits in 3,915 games (**0.077%**). Under the registered stop
  rule as written, any anomaly rejects. The maintainer waived that clause on
  2026-09-01 after the following analysis, and the guard was rate-limited in
  `334c084` so the clause is enforceable in future.
    - All three flagged sides were already decisively lost: round 3 HCEBase at
      -5.32, round 792 HCERefit at -8.72, round 1477 HCEBase at -8.55.
    - The split was 2 baseline / 1 candidate. Reversing all three moves the
      estimate by about 0.3 Elo against a +22.04 result.
    - The guard was added in `d2c7788` and no match had ever run under it.
      Applied to the stored logs it voids nearly every accepted gate,
      including two null calibrations of identical binaries (0.135% and
      0.172%), which is what establishes the forfeits as a harness property.
- **Consequences.** The accepted head fingerprint moves from **6,977,070 /
  2.466** to **7,226,051 / 2.460**; `AGENTS.md`'s behavior-neutral reference
  moves with it. 4.8a (post-refit redundancy removal) and 4.11 (search
  authority on the accepted HCE) are now open. RAR-S70's search counters are
  priors, not a candidate basis, because this refit changed the evaluator the
  search consumes.
