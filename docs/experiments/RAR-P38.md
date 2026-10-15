# RAR-P38 — Compact Dragon UCI names and less work per inference

## Registration before implementation/timing

2026-10-15, maintainer request following F.12; baseline `cf0cb86`, qualified
ordinary executable SHA-256
`a45ea19751ba455886c1a798ff161364017350a734fec4ce6d46e9c8e46cb1d8`.
Work: interface correction, codegen audit, exact implementation and bounded
runtime qualification. Classical remains default. No model/search/scale retune.

Advertise `Evaluation = DragonNNUE`, `DragonEvalFile` and `DragonNNUEScale`.
Accept historical spellings only as unadvertised saved-profile aliases.
Negotiate options from UCI when probing archived baseline binaries; missing or
ambiguous declarations must fail rather than accidentally benchmarking HCE.

## Measured opportunity and competing explanations

F.12 component medians were roughly 0.85–0.92 us with vector kernels. Its
component assembly still shows fourteen software population counts, a
1,536-byte input memset, fresh zeroed change buffers and a 512-byte row-pointer
table per perspective, plus a second full accumulator-read pass to activate.
These are concrete remaining costs in the qualified implementation.

Hypothesis: own reusable scratch, maintain material/pawn metadata with the
already-computed piece deltas, and fuse common delta updates with activation.
The metadata needs neither hardware POPCNT nor additional Board hooks. Expose
eight independent dense output chains to the existing vector kernels if it
fits registers. Preserve the full scalar oracle.

Competing explanations: dense head/memory traffic, hybrid-core migration and
external host load may mask these smaller gains; extra dependencies or register
pressure may offset less bookkeeping. Do not retry F.12's rejected sparse
weight layout without new causal evidence.

## Contract and readiness

- Scratch is allocated with the worker outside nodes and overwritten completely
  before use. HCE's Evaluator carries only a small optional owner, not a large
  inline scratch array. No MaybeUninit, raw-pointer ownership or unsafe Send.
- Each per-king/perspective entry owns exact material and pawn counts alongside
  its piece bitboards. Initialize counts to zero with an empty cold snapshot;
  apply added-minus-removed piece-count changes for each kind. Head selection
  and scaling must reproduce the scalar API after captures/promotions/jumps.
- Keep all eight auxiliary sums, wrapping integer arithmetic, input saturation,
  signed perspective and exact floating reduction/rounding. Every specialized
  fused update must cover both 768-lane halves and the same weights.
- Full 2,738-row fixtures for all supported forced kernels; legal walk/unmake/
  null/special-move/arbitrary-jump/configuration and zero-allocation checks;
  vector intermediate checks, debug/release, both fmt/clippy checks.
- Fresh 5,540 static native comparisons, and unchanged forty-position Dragon
  tree 17,728,094 / EBF 2.604 and default HCE 14,197,472 / EBF 2.557. Any mismatch
  stops acceptance; do not weaken fixtures or change search to restore nodes.

This meets READY_FOR_IMPLEMENTATION for a behavior-neutral runtime change:
existing codegen exposes the waste, exact independent oracles kill mistakes,
and arithmetic/state ownership can be preserved without a Board rewrite.

## Timing protocol and predictions

Retain the maintainer's ongoing shared-host rough-measurement instruction.
Record load and busy-host waiver, keep our own builds/tests off timing runs,
archive ordinary default-feature binaries, no PGO or precision/Elo claim.

Use the unchanged component workload: 256 ordered legal-walk positions,
65,536 calls per kernel, three rotated rounds, exact checksum 3,598,080. Preserve
the parent component executable for a matched reference when screening new code.
Use the existing runner for one parent/candidate/parent `bench 13` A/B/A plus
one candidate Classical control, Hash64/1T/scale72. Consume all forty records
and require identical deterministic fields. Bound component runs at two minutes
and each engine run at fifteen minutes; retain partial/failed attempts.

Prediction: positive component gain, targeting at least 10% without changing
answers. Engine gain may be smaller and host noise may leave it unresolved.
Reject a slower candidate; report inconclusive small engine differences
honestly. Freeze record counts, comparisons and stop rules before exposure.


## Outcome and calibration

**F.13 DONE**, zero games. Compact interface engine `fb18c11`, negotiation
tooling `47c8f26`, further runtime `a8d0892`; all commits stay local on
`dragon_nnue`. The qualified model is unchanged. Ordinary default-feature
release, Rust 1.98.1 / x86_64-pc-windows-msvc, no PGO, SHA-256
`8f1ced1381516fafe0b80b8d9d5b01b2af66b1b44fe5334fff1c00cb0c4a1ecc`. Parent is the already optimized F.12
executable, not the old full scalar backend. The runner's historical arm
labels `scalar-A1/A2` mean that parent here.

| Engine run | Nodes | Time (ms) | NPS | CPU before |
|---|---:|---:|---:|---:|
| Parent A1 | 17,728,094 | 14,064 | 1,260,530 | 10% |
| Further optimized B | 17,728,094 | 12,911 | 1,373,099 | 13% |
| Parent A2 | 17,728,094 | 15,612 | 1,135,542 | 16% |
| Classical control | 14,197,472 | 8,021 | 1,770,037 | 40% |

All 40 Dragon depth/score/node/EBF tuples match in every arm, aggregate
17,728,094 / EBF 2.604; Classical retains 14,197,472 / EBF 2.557. Candidate
NPS / mean parent NPS is **1.146×**. The shared-host
waiver applies to every run. External CPU load, hybrid-core migration, layout
and scheduler noise remain confounds; no precision interval or Elo is implied.

The matched component screen reran the archived F.12 component immediately
after the candidate, without our own concurrent builds/tests. Both use 256
positions, three rotated 65,536-call rounds, checksum 3,598,080 throughout.

| Kernel | Parent median ns/eval | Candidate median ns/eval | Speed ratio |
|---|---:|---:|---:|
| full-scalar | 10,512.2 | 10,738.5 | 0.979× |
| portable | 4,499.9 | 4,611.5 | 0.976× |
| AVX2 | 792.6 | 670.4 | 1.182× |
| AVX-VNNI | 768.5 | 648.4 | 1.185× |

The vector component prediction (at least 10%) passed: both AVX2/VNNI improve
about 18%. Portable and the untouched full scalar control are roughly flat,
with around 2% variation. This supports retaining the combined change, but
does not attribute an isolated gain to any one sub-change. The separate engine
ratio above is the bounded informational search result; the component result
does not imply an equal NPS or strength increase.

## Implementation and audit

- Reuse a worker-owned 2 KiB Scratch allocation for the complete dense input
  and changed feature lists; both input halves and used list prefixes are
  overwritten before use. HCE carries only the small optional owner.
- Delta-maintain exact material/pawn counts in each cached Entry's existing
  alignment padding, avoiding fourteen software population counts. Skip
  unchanged piece kinds; king validation uses the one-bit predicate.
- Specialize six common removed/added shapes; update both 768-lane halves
  and activate them in one AVX2 pass. Larger/cold changes retain the general
  update path. All i16 wrapping, saturation, pairing and byte ordering remain
  exact. Forceable Portable/AVX2/VNNI paths retain the full scalar oracle.
- Eight independent dense output accumulators replace four, improving
  instruction parallelism without changing integer reduction semantics.
- Codegen audit finds AVX2 and VNNI instructions, no FMA, no per-evaluation
  1,536-byte input memset, and one 3,072-byte memcpy guarded by a cold entry.
  That cold bias refresh remains the explicitly measured per-node copy
  exception. No allocation, locks, Arc, Board copy or FEN parsing in inference.
  This is component codegen evidence, not a complete final-executable ISA audit.

Fresh base/AVX2 originals match **5,540/5,540** static records. Both changed-score
controls fail as intended. All 2,738 fixture rows match each supported forced
kernel; legal walks, unmake, nulls, castling/EP/promotions, arbitrary jumps,
head/scale/configuration changes, allocation and worker ownership checks pass.
New vector tests check all fused shapes and fallback with deliberate wrapping
and exact activation bytes. Compact-name UCI smoke plays 16 legal plies over
both evaluators and 1/2 threads; active-search diagnostic guard passes.

All nine final Cargo commands pass: runtime debug/release suites (3 tests each),
runtime fmt/clippy, root debug/release (407/408 passed, zero failed),
root fmt/clippy with all features/targets and ordinary final release build.
Eleven Python harness tests pass. The bundle check before fresh originals
passes 6,194 files and both exact identities. Old/new UCI negotiation is
also exercised against archived engines and the starting static score is 52.

## Reproduce and locate evidence

Local ignored root `analysis/artifacts/dragon-nnue-opt-20261015/`:
`compact-names`, `compact-smoke`, `optimized-followup`,
`component-followup-screen`, `component-parent-followup`, `native-followup`,
`play-followup`, `codegen-followup`, `codegen-followup-audit.json`,
`performance-followup`, `followup-docs-checks`, `followup-manifest.json`.
The manifest hashes sources, binaries and raw logs. Earlier F.12 evidence
is retained in separate directories. Original binaries/model are unchanged.

```powershell
cargo build --release --locked
cargo bench --bench dragon_nnue --no-run
# Archive this ordinary engine and the reported component executable first.
python tools/diag/dragon_nnue_opt.py `
  --baseline analysis/artifacts/dragon-nnue-opt-20261015/optimized-final/rarog.exe `
  --candidate analysis/artifacts/dragon-nnue-opt-20261015/optimized-followup/rarog.exe `
  --micro analysis/artifacts/dragon-nnue-opt-20261015/component-followup-screen/component.exe `
  --reuse-component-screen analysis/artifacts/dragon-nnue-opt-20261015/component-followup-screen `
  --out analysis/artifacts/dragon-nnue-local/further-speed `
  --allow-busy
```

Full-check and fresh-oracle commands are in [usage](../DRAGON_NNUE.md).
`cargo rustc --manifest-path vendor/dragon-nnue/Cargo.toml --release --lib -- --emit=asm`
recreates component codegen. Runtime provenance hashes use normalized LF.
Existing documentation limitations remain: 14 absent historical artifact
references, identical to the baseline, and citation validation needs PR refs
which this checkout lacks. Board, diff, new local links and line endings pass.

Further work should begin with a profile of actual search populations, e.g.
cold refresh frequency, cache reuse and dense-head traffic, before changing
state ownership or introducing another packed layout. No sparse retry, PGO
campaign, new search hook, retune or strength acceptance is part of this close.
