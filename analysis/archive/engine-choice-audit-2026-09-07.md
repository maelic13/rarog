# Basilisk versus Rarog: development choice

Date: 2026-09-07.

> **Historical audit — superseded for current-state guidance.** This document
> records the decision and repository identities examined on the date above.
> Use `GUIDE.md` and `PLAN.md` for current status, sequencing, and model
> assignments. Later work repaired the LazyMargin cache invalidation discussed
> here (`e52074e`, `d07454a`), and the HTML snapshots mentioned below were not
> retained in this repository.

## Recommendation

Keep developing **Rarog**, narrowly, and retain **Basilisk as a frozen reference
and source of reusable ideas, fixtures and tools**, not as a second active engine.

This recommendation is specific to a time-constrained maintainer using coding
agents exclusively and targeting a substantial NNUE transition. It is not a
claim that Rarog currently plays better. The historical playing evidence and
board throughput favor Basilisk. Rarog's implemented invariant checks,
encapsulation, configuration machinery and more complete experiment provenance
are the better starting investment for repeated agent-driven changes.

Confidence is moderate. Future development velocity and NNUE strength have not
been measured. Both are credible foundations; neither has an identified
language-imposed barrier to the target. Changing focus is more important than
the relatively narrow choice between them.

No repository source, plan, settings or commits were changed by this audit.

## Scope and identities

| Item | Audited identity |
|---|---|
| Rarog | `d9d8b26337bc46e0782243cb13dd6781b873c4f1` |
| Basilisk | `da4d1c8b5a25777b0737451d6a798df8def45a41` |
| Shared Net Trainer | `59d190e22162c53efe630938d1609c5baa57d18d` |
| Trainer's pinned Bullet | `cebc78a093d92cbc87e56cfef049184c225270b0` |

Independent source reviews covered each engine's board, search, evaluation,
TT/SMP, UCI/lifecycle, tests, build and CI. The parent review covered experimental
evidence, comparative artifacts, roadmap economics, shared trainer feasibility
and official CCRL data. This was not an exhaustive correctness or licensing
certification.

The execution host identified itself as an Intel Core Ultra 7 165H, not the
user's Ryzen 9 5950X. No fresh timing was represented as Ryzen evidence, and no
games, engine rebuilds or large training jobs were launched. The existing
Ryzen comparison is more relevant than a new laptop microbenchmark.

## What the evidence actually establishes

### Current playing strength: historical edge to Basilisk

Basilisk's `EXPERIMENTS.md:2555-2592` records a 12,000-game four-engine
Colosseum round robin. Its relevant 2,000-game head-to-head was:

**Basilisk 1.10.0dev over Rarog 2.4.0dev: +28.4 +/- 15.3 Elo.**

Conditions were 3+0.03, paired UHO openings, tablebases off and no score-based
draw/resign adjudication. Rarog's
`analysis\node_budget_2026-09-04.md:42-48` identifies this arena as Apple M4.
This is useful evidence for those development builds and that protocol, not a
fresh comparison of today's source revisions on Ryzen or at CCRL time control.
The displayed pool "Elo" had been maintainer-estimated; use the recorded
head-to-head result, not that rating column. The audit did not recover and
independently recalculate this match from its original database.

Neither checked-out head should be presented as fully playing-qualified:

- Rarog's development fingerprint is 7,601,220 nodes / EBF 2.474, but the
  integrated board cluster still owes its playing qualification at 4.11b.17.
- Basilisk contains the prepared 6.5.a rook-scaling candidate, fingerprint
  12,568,898, against Group A's 12,709,666. Its SPRT disposition remains open.

Do not sum accepted patch Elo, difference unrelated oracle experiments, or
compare nominal depth/EBF to manufacture a current strength rating.

### Low-level throughput: real Basilisk advantage

The archived Ryzen 9 5950X comparison used native optimized, non-PGO binaries,
PEXT where applicable, a pinned logical processor and three cyclic rounds.
The five reported medians were independently recomputed from all nine runs:

| Workload | Rarog, M operations/s | Basilisk, M operations/s | Basilisk advantage |
|---|---:|---:|---:|
| Legal generation | 447.131 | 642.646 | 43.73% |
| Capture generation | 98.204 | 120.138 | 22.33% |
| Generation plus make/unmake | 42.521 | 55.031 | 29.42% |
| Start-position perft(4) | 273.741 | 382.726 | 39.81% |
| Two-ply simulation | 351.809 | 513.537 | 45.97% |

Sources: Rarog `analysis\board_audit_2026-09-05.md`,
`analysis\artifacts\board-audit-20260905\manifest.json`, and the complete recipe
in `analysis\board_benchmark_recipe_2026-09-05.md`.

Rarog's board has changed since the original comparison; these are explicitly
historical identities, not a new current-head measurement. A later normalized
SEE comparison reports 44.923 versus 58.335 million calls/s, favoring Basilisk
by 29.86%, but Rarog's round span was 12.20%. The ten measured verdicts match;
that is not full SEE-contract equivalence.

Source: `analysis\see_value_injection_2026-09-07.md:35-100`.

**This does not establish a 44% whole-engine speed advantage.** Rarog's current
full-search profile attributes 6.751% of process samples to generation and legal
delivery. If that entire region improved by 43.7%, with everything else
unchanged, Amdahl's formula gives approximately **2.10% whole-search speedup**:

`1 / (1 - 0.06751 + 0.06751 / 1.437) - 1`.

This is an illustrative conditional calculation across different workloads,
not a forecast of a particular optimization. The profile attributes 24.375%
collectively to generation, make/unmake, check queries and SEE.

Source: `analysis\board_search_profile_2026-09-07.md` and its tracked JSON.

## Engineering comparison

| Dimension | Assessment |
|---|---|
| Board and legality | Both have direct legal generation, redundant board representations, random unwind checks and independent oracles. Basilisk is faster in the archived workloads; Rarog's newer board-v2/SEE contracts are particularly useful for future changes. |
| Search | Both already have serious modern search machinery. Neither needs a feature-list rewrite. Rarog has explicit root-result voting and typed node evidence; Basilisk separates history storage and already wraps searched move transitions. |
| Agent-maintained state | Rarog has the better default containment of mutation and stronger compile-time guardrails. Basilisk exposes its derived board representations publicly; its discipline is enforced mainly by conventions and invariant tests. |
| Experiment safeguards | Rarog binds modern manifests to binary hashes and checks dirty/revision/flavor mismatches. Basilisk has valuable cohort/FEN/contract checks and compiler matching. Both still permit legacy missing manifests, so neither is foolproof. |
| Build/release | Both ship CPU-specific, PGO, x86/ARM assets. Rarog's pinned toolchain, Cargo/xtask integration and ISA verification reduce the supported-toolchain surface. Basilisk's actual Windows production path is MSYS2 Clang, not native MSVC. |
| NNUE readiness | Neither engine contains working NNUE. Both have viable ownership seams, and both can use the same existing engine-neutral trainer. Basilisk's searched-move wrappers are a concrete integration convenience. |
| Development automation | Both CI configurations trigger on master/manual execution, not automatically on the active development/PR path. This is a material weakness for agent-only work. |
| Long-term ceiling | No demonstrated C++ versus Rust limit relevant to top-50/top-100. Data, search quality, inference cost and experimental discipline dominate. |

Rarog is not memory-error-proof: it retains localized unsafe move buffers,
unchecked hot-path accesses, Fathom FFI and probabilistic TT validation.
Both search implementations are large and interaction-heavy. Rust catches
ownership/type failures, not incorrect mate rules, stale caches or bad
experimental designs.

Do not count absent split points or NUMA scheduling as automatic blockers.
Lazy SMP is a credible design, and a 5950X development machine is often most
valuable running many independent single-thread games, not one 32-thread search.

## Concrete source findings

### Basilisk: incoherent shared-TT key/payload publication

`src\tt.h:112-129` loads a separate atomic key fragment, then a separate atomic
payload. Replacement writes payload first and key second
(`src\tt.h:193-206`). A legal interleaving is:

1. Reader matches the old key.
2. Writer replaces the payload with another position's entry.
3. Reader accepts that new payload under the old key.
4. Writer publishes the new key.

Acquiring the old key does not synchronize with the later key publication.
Unlike Rarog's payload-dependent signature, there is no binding of the loaded
key to the loaded payload. Search consumes TT bounds before move legality can
protect it (`src\search.cpp:1325-1331`, `1589-1595`).

This contradicts the comment that mixed pairs are harmless. It is a concrete
atomic-record consistency defect; its frequency and Elo cost were not measured.
It affects SMP, not single-thread search. TSan alone is not a sufficient test:
the operations are atomic, and the defect is logical record coherence.

### Rarog: LazyMargin does not invalidate the evaluation cache

`src\eval.rs:1197-1202` changes the lazy-evaluation margin without clearing the
whole-evaluation cache. The cache is keyed by board hash and halfmove clock
(`1238-1250`), while the margin changes whether important evaluation terms run
(`1311-1336`). Search pushes the current tunable margin at search start.

Cached scores can therefore belong to the previous margin. This chiefly
threatens parameter-sweep/tuning sessions, not fixed-default production play.
Clearing on `ucinewgame` is not a setter-level invalidation contract. This was
confirmed from source; no dynamic reproducer was executed in this audit.

### Basilisk: GCC PGO orchestration does not match GCC output

GCC profile flags exist, but `cmake\pgo-build.cmake:96-115` searches for LLVM
`.profraw` files and invokes `llvm-profdata`. GCC emits `.gcda`.
`CMakeLists.txt:429-462,506-576` therefore advertises a PGO configuration the
orchestration cannot complete. The Clang release path is unaffected.

### Both: mandatory automated gates are weaker than the available tests

Development/PR changes do not automatically run the main CI workflows.
Rarog has broader debug/release/feature coverage; Basilisk has ASan/UBSan and
strong board tests but less complete routine ISA/ARM coverage.
Basilisk's positive Syzygy tests skip through success paths when the hard-coded
local tablebase directory is absent (`tests\test_search.cpp:177-181,800-873`).

Both SPRT runners allow missing legacy sidecars. Rarog's extra provenance
checks are a real advantage, but accepted future gates should require complete
manifests rather than relying on warnings and agent memory.

These are repairable liabilities, not grounds to discard either engine.

### Historical findings were not counted as current defects

The old Rarog search analysis reports a mate-at-halfmove-100 defect. Current
source in **both engines protects checkmate before declaring a rule-50 draw**.
Rarog's predicate is at `src\board\board.rs:992-1011`, with board and depth-4
regressions in `tests\draw_semantics.rs:40-79`. Basilisk's predicate is at
`src\board.cpp:1474-1482`, with board controls and a depth-4 regression in
`tests\test_board.cpp:925-969` and `tests\test_search.cpp:952-961`.
That historical finding was excluded from the current defect comparison.

## The actual CCRL target

The official **40/15, all engines, best versions only** table retrieved in this
audit was dated **September 4, 2026**:

| Boundary/example | Listed engine | Rating |
|---|---|---:|
| Ranks 49-50 | Cataphract 1.5 / Clarity 7.2.0, both 4CPU | 3570 |
| Rank 100 | Ursus 1.0.1, 64-bit | 3443 |
| Rank 2 | Reckless 0.9.0, 4CPU | 3646 |
| Ranks 10-12 | Viridithas 20.0.0, 4CPU | 3633 |

The table mixes each engine's best listed thread configuration; it is not a
strictly single-thread list. Blitz has a separate list. The target moves over
time, and rating uncertainty matters near any cutoff.

Neither "Basilisk" nor "Rarog" appeared in the retrieved 4,706-row all-version
40/15 table. This audit cannot give either a current official 40/15 rating.
Their private-pool numbers cannot be subtracted from these cutoffs.

Reckless and Viridithas are independently confirmed Rust repositories. Their
standing rules out a claimed Rust-language ceiling below the user's target;
it does not predict Rarog's eventual rating.

Primary sources:

- https://computerchess.org.uk/ccrl/4040/
- https://computerchess.org.uk/ccrl/4040/rating_list_all.html
- https://github.com/codedeliveryservice/Reckless
- https://github.com/cosmobobak/viridithas

The HTML snapshots are retained beside this report. Best-list SHA-256:
`e99ca434678be4a0cb26da64a6c8ae8edaecf8efd225de9fa8b695cd8a7dd0d4`.
All-version SHA-256:
`f2be2ce841832f7e2481afce54b70fe9e9059606d871dfb4d903d6993ddd45e0`.
The web-search summaries contained incorrect ratings and repository links;
none of those unverified claims was used.

## Hardware and the shared NNUE constraint

The 5950X is a viable engine-development, CPU-inference, self-play and
regression-testing machine. Use supported AVX2/PEXT paths, retain scalar
reference inference, and measure useful game throughput rather than assuming
32 logical processors equal 32 independent full-performance cores.

**The existing training pipeline does not supply CPU-only training.**
Both plans point to `D:\code\net_trainer`. It already contains a Bullet trainer,
data tooling, a format contract and C++/Rust integer-conformance examples.
The pinned Bullet exposes CUDA, ROCm and Metal backends. With no real backend,
`MockGpu` explicitly errors on kernel launch and GEMM; a successful
no-GPU build is not evidence that training works on a 5950X.

Source:
https://github.com/jw1912/bullet/blob/cebc78a093d92cbc87e56cfef049184c225270b0/crates/gpu/src/runtime/mock.rs

The local trainer's `metal` feature means the M4 is a possible pilot route if
the maintainer deliberately expands its role. Its usable training throughput,
memory limits and correctness have not been established here. It should not
silently become an assumed training farm when its intended role is compatibility.

If no GPU-backed training is permitted at all, changing engines or languages
does not solve the constraint. A CPU-capable training path would need deliberate
implementation/adoption and a measured pilot, with smaller initial experiments.
Approved GPU access is an alternative, not an action taken by this audit.
Inference and training have different hardware requirements.

The present shared network is Chess768 with two perspectives, SCReLU and eight
material output buckets. That is a useful integration baseline, not evidence
that the architecture reaches top-100. Harden splits/checkpoint selection and
quantization parity before scaling. The trainer currently supplies
`test_set: None`; plans for held-out selection are not implemented selection.

## Recommended consolidated direction

Do not require either engine to finish every planned HCE family, broad search
SPSA, small board optimization or release variant before trying useful NNUE.
The shared plans contain valuable contracts, but their accumulated sequence
risks consuming the maintainer's scarce time before the largest intended change.

1. **Freeze the donor and establish one trusted mainline.** Preserve Basilisk's
   exact source, qualified/candidate distinction, binaries and evidence. Do not
   delete branches or artifacts cited by ledgers. No dual release train.
2. **Close correctness and instrument blockers.** Require clean binary-bound
   manifests and automatic active-branch checks. Fix relevant terminal, cache,
   board and publication contracts with independent failing examples. Do not
   disguise repairs as behavior-neutral optimization.
3. **Prove the training route and smallest complete NNUE path.** Run a bounded
   trainer/export/reload pilot, then factual move deltas, per-thread/per-ply
   accumulator state, scalar inference and exact full-refresh parity. Cover EP,
   promotion, castling, king-bucket changes, null, clone, abort and unwind.
4. **Make that network efficient and useful.** Add AVX2 and later ARM SIMD
   against the scalar oracle; grow data and architecture from learning curves,
   independent by-game splits, multiple seeds and frozen evaluation cohorts.
   Preserve score/result/TB-label distinctions. No blanket requirement for
   30-60M positions before the pipeline demonstrates value.
5. **Fit and qualify the complete system.** Revisit score scale, correction and
   pruning interactions under NNUE; use registered paired strength gates,
   longer-time-control and 4T transfer, then an external opponent cohort around
   the actual target. Do not equate beating HCE with reaching a CCRL cutoff.

These are recommendations, not changes to the authorized roadmap. Under the
unchanged Rarog plan, the next executable leaf remains **4.11b.8**, assigned
GPT-6 Astra / Extra High and Claude Opus 5 / High. The 4.12.21 seven-man evidence
obligation remains open until independently verified or explicitly excluded
under its closure contract. Any NNUE-first reordering must change PLAN and
GUIDE together and preserve these obligations visibly rather than silently
skipping or checking them off.

For this decision, the recommended next task is the explicit consolidation and
training-feasibility contract, not blindly executing the old optimization queue.
GPT: **GPT-6 Astra - Extra High**; Claude: **Claude Opus 5 - High**. The work
involves dependency, correctness and experimental-design decisions, not clerical
editing. These are recommendations, not active model-setting changes.

## Agent ecosystem

There is no controlled evidence here that the GPT ecosystem or the Claude
ecosystem produces more accepted engine strength per cost. Do not infer that
from this report's author, a language stereotype, or model marketing.

For the recommended project, sensible defaults are GPT-5.6 Sol / High for
bounded implementation and GPT-6 Astra / Extra High for unresolved architecture
and experimental interpretation. The Claude alternative is Sonnet 5 / High for
implementation and Opus 5 / High for difficult review. Fable 5.1 appears in the
existing Basilisk assignments, but this audit supplies no comparative
measurements for it.

Use one implementer, an independent review of risky changes, mechanical gates
and immutable evidence. Judge agents by accepted changes, rejected defects
caught, maintainer intervention and total cost, not by persuasive explanations
or raw tokens. Better feedback and narrower interventions matter more than
running the most expensive model on every leaf.
