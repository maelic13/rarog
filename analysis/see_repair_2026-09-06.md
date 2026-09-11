# SEE exchange repair — RAR-M28 / 4.11b.5

Completed 2026-09-07 against `e954e38`; engine/test commit `fce0b44`.
Measurements and artifact names retain their 2026-09-06 collection date. This supersedes
the unresolved-defect status in RAR-M27, not its historical observations.
All three named debt tests are now active and pass. No value tuning, HCE
coefficient, search-caller policy or board representation change is included.

## What changed and why

Both SEE kernels select the least-valued **legal** recapturer using the current
exchange occupancy. Each candidate vacates its source before testing its king.
The target remains a ray blocker, but its original captured occupant is excluded
from enemy attackers: its original piece type is no longer present there.
This handles created/released pins, captures along a pin, and falling through
an illegal low-value attacker to a legal higher-value one. The original cached
pin pairs could not describe pins created after pieces left their initial squares.

A king is tested at the target only when it is the selected recapturer. A legal
king capture ends the exchange; the mere presence of an opposing king among
possible future attackers no longer ends an earlier pawn exchange. Attack tests
remain geometric, so a pinned enemy piece still defends a king destination.

Each recapturing promotion contributes both its promotion gain and the new
occupant value. Queen promotion suffices for this material subgame: if captured,
the extra promotion gain cancels the extra victim value, and otherwise queen
promotion gives the greatest gain. Every subsequent target recapture removes
that piece, so its type cannot change that recapture's king safety. This does
not assert that queen promotion is always best in a real tactical position.

Full SEE folds the legal LVA sequence with the existing optional-stop rule.
Threshold SEE follows the same sequence without constructing the gain array:
for positive limit L, `V = max(0, gain - next_V)` satisfies `V >= L` iff
`next_V < gain - L + 1`, provided gain >= L. The result toggles on each such
capture; the +1 preserves equality. This replaces the faulty parity path.
The old immediate moving-piece-loss shortcut was also unsafe when an opponent
could gain an additional promotion bonus. Quiet-aware `Rb2-b1` in the added
fixture loses 1,300, not merely the rook's 500.

Full/ordinary SEE retain immediate-gain shortcuts for ordinary quiets and quiet
promotions. Quiet-aware SEE evaluates ordinary quiets but keeps the quiet-promotion
shortcut. Legal castles still score zero. Initial capture promotions retain the
chosen underpromotion, and en passant removes the victim from its actual square.
Production P/N/B/R/Q/K remains **100/320/330/500/900/20000**. RAR-M29
corrects this document's earlier 32000 sentinel; executable board SEE already
used 20000 during RAR-M28, so no measured result changes.

## Independent evidence

| Case | RAR-M27 full / threshold-zero | Repaired full / threshold-zero |
|---|---|---|
| Rxd5 cxd5 Kxd5 | -400 / true | **-300 / false** |
| Bc6xd5 creates a pin on Nc7 | -230 / false | **+100 / true** |
| Rb2xb1 a2xb1=Q | 0 / true | **-800 / false** |

The original 18-row `tests/data/see-contract-v1.tsv` is byte-unchanged.
`tests/data/see-repair-v1.tsv` adds 23 rows: all 18 color mirrors plus a
promoted piece recaptured at its new value, a pin created on a later exchange,
a pinned knight skipped for a legal rook, a quiet move allowing promotion,
and an initial king capture. Python-chess independently enumerates every legal
same-square reply and all four promotion choices. Explicit expected arithmetic
must agree before writing fixtures; it is not derived from Rarog output.

Rust checks all 41 rows, threshold boundaries including -301/-300/-299/0,
both threshold policies, unchanged FEN and internal state consistency. The
three historical debt tests separately remain active. Four deterministic legal
walks additionally check full/threshold parity on **1,802 captures**; parity
complements independent truth and does not replace it. These fixtures do not
turn LVA SEE into exhaustive tactical minimax on arbitrary branching positions.

## Verification, fingerprint and limits

The complete default-feature suites pass: **268 debug / 269 release**, zero
failed or ignored in 26 test-result groups per profile. The extra release test
is the existing release-only block in `tests/fuzz_lite.rs`. Six Python tests
pass, including a deliberately wrong arithmetic negative control and checking
all four recapture promotions when the promoted piece is captured again.
`cargo fmt --check` and `cargo clippy --all-features --all-targets -- -D warnings`
exit zero. The production binary was rebuilt AFTER these checks with
`cargo build --release --no-default-features`, with no RUSTFLAGS and no texel.

Production `bench 13`: **7,601,220 nodes / EBF 2.474**, previously
**6,901,489 / 2.458**. This is **+10.14% nodes**, not a strength or throughput
result. It is the development comparison baseline for 4.11b.6 onward; the
playing cluster remains unqualified until 4.11b.17. The last strength-qualified
baseline is unchanged. No matches, tuning or comparative NPS study ran here.

Changed SEE answers can reach all production consumers inventoried in RAR-M27:
ProbCut admission, capture pruning, qsearch filters, full and staged ordering,
LMR eligibility and capture-history classification. The optional quiet-aware
prune is still default-off. No per-consumer causal measurement is claimed;
4.11b.7 profiles cost, 4.11b.11 optimizes this corrected kernel, 4.11b.17 gates
the cluster, and 4.11b.18 refreshes affected endgame evidence. In particular,
do not compare later optimizations against the old faulty SEE semantics.

**Invalid invocation retained explicitly:** passing `bench 13` as process
arguments emitted only the startup banner and exited zero. That attempt did
not run a benchmark. `checks.json` marks it invalid; `bench13-argument-noop.log`
preserves the evidence. The replacement uses `bench_counters.run_bench` to keep
UCI stdin open until the summary, requires nodes > 0 and exactly one EBF, and
checks the actual engine exit. It measures the hash-verified immutable local
copy, so no additional rebuild was needed after detecting the invocation error.

## Reproduction

`analysis/artifacts/see-repair-20260906/` contains commands and direct statuses,
full test logs, the valid bench transcript, compiler/features/source/binary
identity, file hashes, and `engine.patch.gz` (exact engine diff against
`e954e38`, recoverable with Python `gzip.decompress`). The production SHA-256
is `C87F0063983C32ADE8EE771EB89C2D50DD32921653772F1B9B02FEA6C98F2347`.
Use the committed repair or apply the archived diff to that baseline; do not
copy another engine's SEE values. From the repository root:

```powershell
python tools/diag/see_contract_oracle.py
python -m unittest discover -s tools/diag -p test_see_contract_oracle.py
cargo test -- --nocapture
cargo test --release -- --nocapture
cargo fmt --check
cargo clippy --all-features --all-targets -- -D warnings
cargo build --release --no-default-features
Copy-Item target/release/rarog.exe target/rarog-see-repair.exe
python analysis/artifacts/see-repair-20260906/bench_recipe.py
```

The archived bench recipe intentionally requires the recorded binary hash.
A different compiler/build configuration needs its own identity record and
must independently reproduce the fingerprint; do not weaken the archived check.
