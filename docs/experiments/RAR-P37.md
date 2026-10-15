# RAR-P37 — Exact incremental Dragon inference and CPU kernels

## Registration, before optimized code or timing

2026-10-15; maintainer-requested F.12. Primary work: audit, implementation,
deterministic and bounded performance qualification. Baseline branch state is
`8356cc3` plus the separately committed exact embedded-asset packaging step.
The frozen model is 70,136,671 bytes, SHA-256
`c6e25a7fde5971dc47e78e19764c82b8d18a438bbd5ee4da065d061115064417`.

Opportunity measured by RAR-P36: scalar Dragon 208,408 NPS versus Classical
1,796,011, with different search trees. Code audit identifies two full
32-piece/1,536-lane transformer refreshes per cache miss; scalar first-layer
pair products; output-major float rows; input serialization/validation per
node. No search or model change is proposed.

Competing explanations: search tree mix, background scheduling and dense-head
cost may dominate the observed gap. Transformer deltas alone might therefore
give a smaller engine gain than kernel gain. Cold king entries and arbitrary
position jumps may reduce reuse. Memory footprint/dispatch can offset gains.

## Contract and design

- Package the exact model under `assets/nnue/` and embed it as `<embedded>`;
  Classical remains default and explicit file overrides stay hash-checked.
- Retain `Network::evaluate` as an independent full scalar reference.
- Per-worker lazy accumulators are keyed by perspective and exact king square.
  Store the last twelve piece bitboards; subtract removed features and add new
  ones on demand. This handles skipped evaluations, null, arbitrary jumps,
  unmake, captures, castling, en passant and all promotions without Board hooks
  or copying 3 KiB at every ply. Initialize each cold entry once; reset validity
  outside nodes when configuration changes. Cache all eight auxiliary heads.
- Own about 408 KiB per active worker; allocate only while configuring Dragon.
  HCE needs no accumulator allocation. Never share mutable accumulator state.
  A cold entry's single 3 KiB bias initialization is an explicit measured copy
  exception: it establishes the transform baseline; warm evaluations use deltas.
- Use runtime CPU dispatch selected at cache construction: portable, AVX2 and
  AVX-VNNI where available. Every unsafe vector access is fixed-width and
  bounded, with local safety explanations. Base binaries remain portable.
- Preserve wrapping i16 updates, saturating pair semantics, wrapping i32 sums,
  pawn-count scaling and perspective. Activations are at most 126, so signed
  i16 pair saturation cannot fire and non-saturating VNNI dot products agree.
- Transpose float second-layer weights at load; vectorize across output
  neurons, preserving every neuron's original four-lane reduction tree and
  separate multiply/add rounding. No FMA contraction, reassociation or retune.

Runtime intrinsic contracts checked against Rust's primary documentation:
[AVX2 add](https://doc.rust-lang.org/core/arch/x86_64/fn._mm256_add_epi16.html),
[unsigned/signed pair products](https://doc.rust-lang.org/core/arch/x86_64/fn._mm256_maddubs_epi16.html),
[VNNI](https://doc.rust-lang.org/core/arch/x86_64/fn._mm256_dpbusd_avx_epi32.html),
[runtime detection](https://doc.rust-lang.org/std/macro.is_x86_feature_detected.html).

## Invariants, falsifier and stop

All 2,738 frozen rows must match, including every supported forced kernel,
cold/warm/cleared state and changing heads/scales. Compare random legal walks,
unmake and arbitrary FEN jumps against the untouched scalar evaluator. Require
zero node allocations and legal 1T/2T UCI play. Fresh base/AVX2 Dragon probes
must match all 5,540 records with changed-score controls rejected. Default HCE
bench remains 14,197,472 / EBF 2.557; Dragon must retain 17,728,094 / EBF 2.604
and every position's depth/score/nodes. Any mismatch stops timing acceptance.
Debug/release suites, formatting and clippy must pass. No games or Elo claim.

## Bounded timing protocol and prediction

Build/archive ordinary default-feature release binaries, no PGO. The existing
shared-host instruction (host cannot be kept idle; run rough estimates anyway)
is retained for this ongoing requested NNUE optimization. Record CPU/process
checks and the waiver explicitly; never present precision from these reads.

Run a component benchmark with identical ordered legal positions and checksums:
full scalar, incremental portable, incremental AVX2 and incremental VNNI when
supported, three rotated rounds of 65,536 evaluations after one untimed
correctness pass. The versioned benchmark freezes its four legal-walk seeds,
position order and xorshift generator. This
separates accumulator reuse from vector kernels; setup/loading is excluded.
Then one engine `bench 13` for the packaged scalar baseline and optimized
Dragon, followed by one scalar baseline repeat (A/B/A), plus one optimized
Classical bench to verify its fingerprint. Consume all 40 rows per run, using
the existing parser. Bound each engine run at 15 minutes, each micro run at two
minutes. Keep logs/hashes/partial failures. This is explicitly an informational
screen, not PROCESS's pooled-PGO precision claim. Leave a repeatable runner.

Prediction: exact outputs and tree fingerprints; incremental portable faster
than full reference; vector kernels faster still. Target at least 2× engine
NPS over the scalar baseline for this large opportunity. A smaller/no gain
triggers a bottleneck review rather than a speed claim. No precise magnitude
or playing-strength prediction. Keep only optimizations that survive parity
and this bounded screen; residual tuning is separately reported.

## Audit screen and second candidate, before its timing

The first component screen retained exact checksums for all 256 legal-walk
positions and three rounds of 65,536 calls. Medians were about 10.50 us for
full scalar, 9.76 us for incremental portable, 0.867 us for AVX2 and 0.819 us
for VNNI. Thus scalar head work masks much of the portable delta benefit;
vectorizing the head is essential. This is component evidence under load,
not an engine NPS claim. Raw: `component-screen/`.

The second candidate fuses common fixed-size feature deltas so each vector
is loaded/stored once without variable-length inner loops. VNNI weights are
also packed at load as four inputs by sixteen outputs; zero four-input
blocks skip their weights, and four independent dot chains hide latency.
The original output-major rows remain the scalar reference. Transposition
changes representation only. Require the same forced-kernel/intermediate
checks and compare the identical micro corpus again before retaining it.


### Sparse packing rejected

The second screen remained exact, but packed sparse VNNI was slower than
AVX2 in every round (medians 1.534 us versus 0.921 us); the previous dense
VNNI path had been slightly faster than AVX2. Background load also slowed
the scalar controls, so absolute cross-run deltas are not claimed. The
within-run loss is sufficient to reject this representation/branching design.
It is removed; the dense four-output VNNI kernel and fused small updates
remain for final qualification. Raw `component-screen-v2/` and the complete
`rejected-sparse-source/` snapshot preserve the failed candidate. Prediction
calibration: sparse layout/zero skipping did not help this corpus and CPU;
no retuning or accuracy relaxation was used to rescue it.


### Final candidate before its qualification/timing

Retain dense VNNI and fused common deltas. For the portable cached path only,
replace inactive pair saturation with an ordinary wrapping integer dot:
2*126*128 = 32,256 is within i16. This exposes a simple Rust iterator reduction
to LLVM's baseline SIMD auto-vectorizer. The independent full scalar path keeps
its original saturating pairs. All first-layer results still convert to f32
at the same point, then use the original floating reduction. The final screen
must qualify this portable path independently, including arbitrary jumps,
all frozen rows, integer wrap controls and zero allocations.


## Final result and disposition

**F.12 DONE for the requested exact runtime optimization.** Asset packaging
`0624717`; optimized engine `75dde4f`; benchmark tooling `6881550`.
All commits stay local on `dragon_nnue`. Original model bytes are unchanged.
Final ordinary release, Rust 1.98.1 / x86_64-pc-windows-msvc, default features,
no PGO; candidate SHA-256 `a45ea19751ba455886c1a798ff161364017350a734fec4ce6d46e9c8e46cb1d8`. Scalar packaging
baseline SHA-256 `1675dabfbd52c2a64db59a6ffb702d22c95e2e57880a9f6e1ccd8d0acca3ce46`.

| Engine run | Nodes | Time (ms) | NPS | CPU before |
|---|---:|---:|---:|---:|
| Scalar A1 | 17,728,094 | 77,656 | 228,290 | 25% |
| Optimized B | 17,728,094 | 14,723 | 1,204,108 | 14% |
| Scalar A2 | 17,728,094 | 126,590 | 140,043 | 12% |
| Classical control | 14,197,472 | 10,273 | 1,382,018 | 31% |

All three Dragon runs have identical per-position depth, score, nodes and EBF,
40/40, aggregate 17,728,094 / EBF 2.604. The HCE control preserves
14,197,472 / EBF 2.557. Optimized NPS is **6.54×**
the mean of the two scalar reads. This is a shared-host A/B/A screen with an
explicit waiver, not a pooled-PGO interval or an Elo estimate. Baseline timing
spread and hybrid-CPU scheduling remain confounds; no precise small gain is
claimed. The prospective 2× engine target passed without changing the tree.

Component medians on the final 256-position corpus, three rotated rounds of
65,536 evaluations each (all checksums 3,598,080):

| Kernel | Median ns/eval | Speedup over full reference |
|---|---:|---:|
| full-scalar | 12,006.5 | 1.00× |
| portable | 5,139.9 | 2.34× |
| AVX2 | 855.0 | 14.04× |
| AVX-VNNI | 920.4 | 13.04× |

AVX2/VNNI ordering varies across these noisy rounds. Automatic dispatch uses
dense VNNI when supported, then AVX2; both remain explicitly forceable for
conformance and attribution. No claim of a resolved small difference between
those ISAs is made. Portable first-layer simplification materially improves
its scalar control while retaining an independent saturated-pair reference.

All 2,738 fixtures match for each supported forced kernel. Fresh original
processes match 5,540/5,540 static records with two changed-score controls
rejected. Legal walks, unmake, null, special moves, arbitrary jumps, changing
heads/scales, reset/configuration, helper ownership and zero-allocation tests
pass. Root and runtime debug/release suites, both fmt checks, both clippy
checks and ten Python parser/negative tests pass. The final embedded executable
plays sixteen legal UCI plies across both evaluators and 1T/2T; stop responsiveness
remains qualified. Bundle verification passes before original comparisons.

Standalone component assembly shows AVX2 pair dot and VNNI dot instructions,
no FMA instructions, and exactly one memcpy in cached evaluate: the guarded
3 KiB cold-bias initialization. There is no warm large-copy path. This is a
component codegen audit, not a cross-platform/final-binary ISA certification.

Calibration: exactness and large vector/engine gain predictions passed;
incremental portable alone was insufficient before its head simplification;
sparse VNNI packing failed and was removed. Strength, tuned margins, PGO pool
precision and non-host SIMD performance remain separate questions. No model,
training, eval semantics, search policy or HCE parameters changed.

## Reproduction and raw evidence

Build the scalar packaging baseline from `0624717` in a separate checkout with
`cargo build --release --locked`; build the optimized candidate the same way.
`cargo bench --bench dragon_nnue --no-run` prints the component executable path.
Pass these three executables to the committed runner:

```powershell
python tools/diag/dragon_nnue_opt.py `
  --baseline <packaged-scalar.exe> --candidate target/release/rarog.exe `
  --micro <component-executable> --out analysis/artifacts/dragon-runtime-new
```

The idle guard stays on by default. The registered run used `--allow-busy`
for the maintainer's shared-host constraint. `--reuse-component-screen` accepts
an archived component run only when its binary and every runtime source hash
match; changed-binary/source controls are tested. No speed number here comes
from an all-features or Texel binary.

Raw root: `analysis/artifacts/dragon-nnue-opt-20261015/`. `asset-final/` holds
the packaged scalar build and empty-directory smoke; `optimized-final/` holds
all nine final commands/logs and the measured candidate. `native-probe/` and
`play-smoke/` retain all original/UCI evidence; `codegen/` retains assembly.
`component-final/` and `performance/` retain every timing/checksum/position,
CPU check and binary identity; earlier screens, failed lint logs and the rejected
sparse source remain preserved. `manifest.json` hashes canonical source and raw
qualification outputs. Large reproducible compiler caches stay ignored.

Board generation, diff and changed-link checks pass. Full check_guide retains
the same fourteen missing historical artifacts as the starting checkout;
check_citations cannot run without that clone's PR refs. These pre-existing
provenance limitations do not qualify as passed engine checks.
