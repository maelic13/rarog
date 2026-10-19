# RAR-P40 — Exact Fritz HCE throughput

Registered in the reconstruction project as HCE-002 on 2026-10-15;
resumed and mirrored here 2026-10-19. Primary kinds: neutral implementation,
deterministic qualification and informational performance qualification.
Baseline dragon_nnue HEAD 3de5d3d; archived ordinary release SHA-256
510de2da284ea29edafe1abb7b75935795f25e43e1dacd6c9d693cb07f585545.
The maintainer requested no commits or pushes.

Two prospectively bounded causal families: remove the recovered full-eval
PC/match dispatcher using assembly-verified structured control flow, then
incrementally update position inputs from changed bitboards. The full-eval
CFG has noncrossing loops and header-only entries; the pawn routine's crossing
intervals retain the original dispatcher. Translation retains every original
instruction, width, parameter and branch edge. No guessed CFG repair.
Incremental preparation clears all removals before additions (capture and
promotion replacement), keeps wrapping PST/signature/key state, and handles
arbitrary position jumps, turn changes, undo, rights changes and cache resets.
The full-rebuild implementation remains as an independent test reference.

Prediction: less dispatcher overhead and fewer unchanged-piece table reads.
Alternatives: table/cache traffic dominates, extra delta bookkeeping costs
more, and shared-host load confounds small engine timing differences.
No model, score, scaling, native Classical, Dragon, search or option changes.
No per-node allocation, locks, unsafe code or Board copies.

Acceptance/falsifier: 486 native snapshots, cold/warm/reset, parameter negative
control, incremental/full preparation differential, adapter/worker/allocation
checks, mandatory full debug/release, fmt and zero-warning all-target/all-feature
Clippy. All forty (position, depth, score, nodes, EBF) tuples must match the
archived baseline separately for every backend. Classical fingerprint
14,197,472 / 2.557; Dragon 17,728,094 / 2.604, scale 72.
Keep clear causal gains; reject any semantic difference or regression.

Measurement: existing dragon_nnue_bench.py parser and Session, bench13,
Hash64, 1T, no Ponder, embedded Dragon at scale72, every backend. Component:
486 positions, 1,990,656 calls per round, rotations 0/137/317 and checksum
-2,494,464 each round. Stop after the two causal screens and final archived
parent/candidate/parent engine reads; no own builds/tests during timing.
Bound each engine at fifteen minutes. The initial run stopped at CPU11%;
the maintainer explicitly authorized informational shared-host timings.
Every resumed manifest records the waiver and host load/process inventory.
These are ordinary-release informational reads, with no pooled-PGO confidence,
performance equivalence or Elo claim. Raw records in the reconstruction
work/hce-speed-20261015 folder are copied to this repository's ignored
analysis/artifacts/fritz16-hce-speed-20261019 at completion.

Completed 2026-10-19. Retained direct control flow for the 164-block full
evaluator and incremental position preparation; the 209-block pawn routine
retains its proven dispatcher because its assembly-order intervals cross.
No parameters, arithmetic widths or search behavior changed.

Component parent/candidate/parent medians: 379.4 ns to 197.0 ns
per evaluation (1.93x throughput), across the frozen 486-position
corpus, three rotated rounds per arm, exactly -2,494,464 checksum each round.
The initial isolated screens measured 407.6 ns baseline, 233.1 ns after direct
control flow, and 217.9 ns after incremental preparation. These screen values
and the final interleaved values measure different host-load samples.

Final ordinary-release bench13, Hash64, 1T, embedded Dragon scale72:

| Backend | Parent pooled NPS | Candidate NPS | Change |
| --- | ---: | ---: | ---: |
| Classical | 1.728M | 1.778M | +2.9% |
| Fritz16HCE | 1.424M | 1.591M | +11.7% |
| DragonNNUE | 1.234M | 1.301M | +5.4% |

Each parent/candidate/parent arm consumes all forty records for every backend;
all (position, depth, score, nodes, EBF) tuples agree exactly. Parent pooled NPS
uses summed nodes/time from the two parent arms. Shared-host timings are
informational with the explicit maintainer waiver recorded in every manifest;
backend variation is a confound, not evidence of a Classical/Dragon change.
No pooled-PGO confidence interval, performance-equivalence or Elo claim.

486 direct native snapshots, independent preparation/default adjustment,
cold/warm/reset and the material-parameter negative control pass. The new
incremental/full rebuild test checks both turns, forward/reverse position
jumps and periodic resets. Full Rarog debug/release suites, adapter special
moves/undo/null, worker/mode-switch and zero-allocation tests pass; formatting
and all-feature/all-target Clippy pass at zero warnings. Assembly checks find
no allocation/memcpy/memmove calls in the four hot HCE functions. Bundle
verification passes all 16,936 files and six unique originals.

Candidate SHA-256: 3c37e72b8f49944550f7a0be517ebee0ab9173f7264326cd08ef7a315d0c2c61.
Ready binary: target/release/rarog-fritz16-hce.exe in Rarog. No commits or pushes.
Raw records, source/binary/log hashes and checks: qualification.json under
analysis/artifacts/fritz16-hce-speed-20261019 in Rarog and
work/hce-speed-20261015 in the reconstruction checkout.

Calibration: the dispatcher cost hypothesis is supported by the large first
component gain. Incremental preparation adds a smaller gain even for unrelated
FENs; the exact engine read qualifies interaction with search. Host scheduling
limits precise engine effect estimates. Stop rule reached after two causal
families; further work needs a newly measured hotspot. Original whole-engine
semantic/nondefault-option/lazy-path obligations remain open.

PLAN/GUIDE synchronization passes in both repositories. Rarog's broad
documentation guard reports the same fourteen missing historical artifact
paths as before this work; each reference is mechanically confirmed in
unchanged HEAD documents. This remains a pre-existing documentation
limitation, not a passed broad documentation guard.
