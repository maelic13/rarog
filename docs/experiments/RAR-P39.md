# RAR-P39 — Further Dragon throughput screens and aligned dense rows

## Prospective registration, 2026-10-15

Maintainer requests another bounded DragonNNUE speed pass. Primary work is
codegen diagnosis, exact implementation, deterministic and runtime qualification.
Baseline `e058052`, runtime `a8d0892`; ordinary release SHA-256
`8f1ced1381516fafe0b80b8d9d5b01b2af66b1b44fe5334fff1c00cb0c4a1ecc`.
F.13's qualified component is
`analysis/artifacts/dragon-nnue-opt-20261015/component-followup-screen/component.exe`.
Model, score scale, UCI names, Board/search hooks and HCE stay unchanged.

## Evidence, mechanism and alternatives

The saved F.13 component assembly `codegen-followup/release/deps/dragon_nnue-*.s`
shows the first layer extracting eight scalar partials for each of sixteen
outputs: repeated GPR extraction/adds, individual float conversion and saved
callee registers. The AVX2 finish function calculates fifteen square/linear
activations with scalar instructions and has 1,128 bytes of stack. Common
fused delta kernels still call generic array-from_fn helpers at the ISA boundary.
These are concrete removable operations in this engine, with no search change.

H1: reduce four output vectors together with horizontal integer SIMD adds,
then vector-add bias and convert four i32 sums to f32. Integer addition may
reassociate modulo 2^32; CVTDQ2PS must follow the complete biased sum.
H2: explicitly vectorize the fifteen independent square/linear activations.
Use overlapping eight-value chunks at offsets 0 and 7 so no masked/out-of-range
memory access or extra lane enters the 30-input float layer.
H3: replace the feature-row array callback with checked fixed-array filling,
allowing common delta paths to inline row lookup. Screen this separately if
needed. None is another sparse weight layout; RAR-P37's failed retry stays held.

Competing explanations: memory/transformer traffic dominates; horizontal SIMD
dependencies, larger code, unaligned loads or ABI costs can offset saved scalar
instructions. Hybrid scheduling/external host load can mimic small gains.
The first two changes are ready for implementation because existing exact
intermediate tests plus the full scalar/native oracles directly falsify errors.
Any later adjustment needs its own measured/codegen cause, recorded here first.

## Invariants, falsifiers and scope

- Keep exact i16/i32 wrapping, input saturation/order, bias-before-conversion,
  float operation/reduction order, skip/auxiliary paths and final scaling.
  No FMA, relaxed math, approximation, retune or model substitution.
- All SIMD memory accesses stay within initialized arrays/slices. No new
  allocation, locks, Arc, Board copies or node hooks. Retain runtime dispatch
  and forceable Portable/AVX2/AVX-VNNI plus independent full scalar reference.
- Existing adversarial intermediate tests compare integers/float bits. Extend
  activation checks to cover negative/zero/extreme values and overlapping lane
  7; all 2,738 fixtures per supported kernel, legal/unmake/null/special/jump/
  config/head transitions, worker and allocation checks must remain exact.
- Fresh 5,540 native static pairs, changed-score negative controls, 16 legal
  UCI smoke plies at 1/2T, all forty Dragon tuples and both aggregate
  fingerprints unchanged: Dragon 17,728,094 / 2.604, HCE 14,197,472 / 2.557.
- Final runtime and root debug/release tests, both fmt/clippy checks; normal
  release archive only. Any mismatch stops acceptance, without relaxing tests.

## Frozen timing and stop rule

Reuse unchanged 256-position, 65,536-call, three-rotated-round component workload
and exact checksum 3,598,080. Compare archived parent and each screened candidate;
bound every component process at two minutes, no own concurrent builds/tests.
At most three causal candidate screens, retaining snapshots and failed output.
Keep a positive component gain; reject a clear regression. Target 5–15%, but a
smaller unresolved result may close NO_CHANGE. No endless sweep or sparse retry.

For an accepted component candidate run the existing final parent/candidate/
parent engine A/B/A, plus Classical control: bench13, Hash64/1T/scale72, all forty
records, fifteen-minute cap per engine process. Reuse the final identical-source
component screen via its source/binary guard. Preserve every raw transcript,
CPU reading, source/binary/model hash and partial failure.

The maintainer's shared-host rough-measurement waiver applies; record it in
every manifest, keep our own load off timers. No precision interval, Elo,
PGO campaign, external profiler or strength gate is requested. Engine gains
may remain unresolved even when component gains are clear.


## Screen 1 calibration and candidate 2

Candidate 1 (SIMD reductions + activation) passes exact intermediate tests and
all forced-kernel adapter checks. Codegen removes many scalar instructions
(VNNI forward 226 → 85, finish 337 → 226), but the matched component screen
does not improve: VNNI 1.001×, AVX2 0.966× parent; scalar controls vary 1–2%.
Reject that combination in isolation. Hypothesis missed throughput, not
correctness or emitted instructions. Memory/ABI/latency can dominate instruction
count; do not declare speed from smaller assembly. Snapshot and logs retained.

Candidate 2 starts again from the parent and isolates H3: checked fixed-array
row filling instead of array-from_fn callbacks. This is the already registered
mechanism; no arithmetic or test change. If positive, the remaining third screen
may combine it with H1/H2 to determine whether those add value after lookup cost
is removed. Every final accepted source must still meet all original gates.


## Screen 2 calibration and prospectively selected final candidate

Checked row filling alone also fails the screen: AVX2 0.940× and VNNI 0.935×
parent. Portable varies 4%, so host noise is material; no gain is established.
Keep both snapshots and reject this source. Do not combine two unproven changes.

Before selecting the final screen, a copied-source, configuration-only address
diagnostic in Dragon work/nnue-alignment-20261015 inspected the actual loaded
buffers. FT was already 64-byte aligned, while integer head bases had residues
48/0/16/32 and transposed float rows 56/24 modulo64. This is one diagnostic
allocation, not proof of every engine allocation, but the current types promise
only byte/f32 alignment. Many 32-byte loads can straddle a cache line.

Final candidate: start from the unchanged parent; align dense signed-byte rows
and the transposed float matrix to 64 bytes. Use safe Vec-owned initialized
padding with an aligned borrowed slice for bytes, and a repr(align(64)) wrapper
for the inline float matrix. No raw ownership, unsafe allocator or FT copy.
Up to 63 extra bytes per integer head; small float-field/Head padding and a
small extra indexing cost. Serialized model hash/values/order, all arithmetic,
scalar oracle, dispatch, node ownership and every original gate remain unchanged.
This mechanism changes load alignment, not sparsity or score calibration.

Prediction before screen3: positive but modest, 3–10% vector component gain;
initial 5–15% scalar-instruction hypothesis is rejected. Risks are offset/bounds
checks, layout effects and external load. Require byte-for-byte storage tests,
pointer alignment, complete kernel/native parity and registered engine A/B/A.
If this last candidate is flat/slower, restore parent runtime and close
NO_CHANGE; do not extend the three-screen budget.


## Final qualification and disposition

**F.14 DONE, alignment candidate retained**, engine `d56f6d3`; no games.
Two scalar-instruction/callback hypotheses were rejected and their source
snapshots retained. Only dense-row alignment ships: safe initialized Vec
padding for i8 weights and an aligned type for transposed float rows. No new
SIMD arithmetic, model, scale, search policy or UCI change. Initial 5–15%
instruction-count prediction failed; the separately frozen alignment prediction
is supported by 6.0% VNNI / 3.3% AVX2 component gains on this host.

Final ordinary default-feature build, no PGO, Rust 1.98.1 / Windows x64 MSVC,
SHA-256 `01fbb7f9b4909bf91cc0790208e9eaf269119dfceb00a6c1f995cb906f518445`. Baseline is F.13's already optimized
`8f1ced1381516fafe0b80b8d9d5b01b2af66b1b44fe5334fff1c00cb0c4a1ecc`.
The runner's historical `scalar-A1/A2` labels mean this optimized parent here.

| Engine arm | Nodes | Time (ms) | NPS | CPU before |
|---|---:|---:|---:|---:|
| Parent A1 | 17,728,094 | 14,015 | 1,264,937 | 21% |
| Aligned B | 17,728,094 | 14,122 | 1,255,352 | 11% |
| Parent A2 | 17,728,094 | 14,766 | 1,200,602 | 11% |
| Classical control | 14,197,472 | 8,475 | 1,675,217 | 9% |

All forty depth/score/node/EBF tuples match in all Dragon arms, aggregate
17,728,094 / 2.604. Classical preserves 14,197,472 / 2.557. The observed engine
ratio is **1.018×** mean parent NPS. Parent spread,
external load and hybrid-core scheduling limit this rough shared-host screen;
no small engine gain, interval or Elo is established. The maintained busy-host
waiver is in the raw results; no own builds/tests ran during timers.

The final component is the identical-source screen reused by hash guard:

| Kernel | Parent median ns/eval | Aligned median ns/eval | Throughput ratio |
|---|---:|---:|---:|
| full-scalar | 10,344.4 | 10,322.0 | 1.002× |
| portable | 4,295.3 | 4,586.9 | 0.936× |
| AVX2 | 670.8 | 649.7 | 1.033× |
| AVX-VNNI | 643.0 | 606.6 | 1.060× |

All use 256 positions, three rotated rounds of 65,536 calls, checksum 3,598,080.
The full scalar control is flat. Portable is 6.4% lower with substantial
round variation; that is an unresolved performance limitation, not an accepted
portable speed claim. Retention is for the positive AVX2/VNNI component screen
on this host, whose automatic dispatch chooses VNNI. Non-host platform timing
and portable-only engine timing remain unqualified. No further screen was run
after the pre-registered three-candidate cap.

All 2,738 fixtures match every supported forced kernel. Fresh original base
and AVX2 diagnostics match 5,540/5,540, both changed-score controls rejected.
Legal walks/unmake/nulls, arbitrary jumps, castling/EP/promotions, material/head
and scale/config changes, worker ownership and zero node-allocation tests pass.
UCI smoke plays sixteen legal plies across both evaluators and 1/2T and passes
the active-search diagnostic guard. Storage tests preserve every byte and test
64-byte alignment across short/full byte buffers and all transposed rows.

All nine final Cargo commands pass: runtime debug/release (4 tests each),
runtime fmt/clippy, root debug/release (407/408 passed, zero failed),
root fmt/clippy all features/targets, final ordinary release. Eleven Python
harness tests pass. Bundle verification before original execution passes 6,194
files, both original identities and section/model ranges. Component codegen
contains both AVX2/VNNI, no FMA, and one guarded 3 KiB cold-bias memcpy in cached
evaluate; no per-node allocation or new large copy. Scalar arithmetic and the
serialized model remain unchanged, despite the loader's owned-storage change.

## Reproduction and raw paths

Local ignored root: `analysis/artifacts/dragon-nnue-simd-20261015/` contains
`parent-x86.rs`, `parent-lib.rs`, rejected `candidate-1-x86.rs` and
`candidate-2-x86.rs`, all `component-1/2/3` and `parent-component-1/2/3` screens,
`codegen-1`, final `optimized-final`, `codegen-final`, `codegen-final-audit.json`,
`native-final`, `play-final`, `performance-final`, `docs-checks` and
`manifest.json`. The address diagnostic's copied source/output stays at
`D:/code/dragon/work/nnue-alignment-20261015/`. No original bytes were modified.

```powershell
cargo build --release --locked
cargo bench --bench dragon_nnue --no-run
# Archive the ordinary engine and the reported component executable.
python tools/diag/dragon_nnue_opt.py `
  --baseline analysis/artifacts/dragon-nnue-opt-20261015/optimized-followup/rarog.exe `
  --candidate analysis/artifacts/dragon-nnue-simd-20261015/optimized-final/rarog.exe `
  --micro analysis/artifacts/dragon-nnue-simd-20261015/component-3/component.exe `
  --reuse-component-screen analysis/artifacts/dragon-nnue-simd-20261015/component-3 `
  --out analysis/artifacts/dragon-nnue-local/alignment-speed `
  --allow-busy
```

Final checks and native probe commands are in [usage](../DRAGON_NNUE.md).
Manifest hashes source and raw outputs; runtime provenance distinguishes the
upstream source from current loader/storage. Existing fourteen absent historical
artifact references and missing PR refs remain documentation limitations;
board/diff/new local links and line endings pass. All commits stay local;
ordinary next roadmap work is C.7. Further NNUE speed work should profile actual
search update populations and portable codegen before another representation
change; this close promises neither a resolved search gain nor stronger play.
