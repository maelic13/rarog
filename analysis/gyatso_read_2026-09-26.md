# GyatsoChess read, 2026-09-26

A reading of https://github.com/GyatsoYT/GyatsoChess (commit `75d38ad`,
2026-09-26) to answer three maintainer questions: whether Nim is a better
engine language than Rust, whether Gyatso has mechanisms Rarog lacks, and
whether its strength is mostly the network. Gyatso is not a donor
(`PROCESS.md`, *The independence boundary*); this note records mechanisms
and a calibration point, and PLAN owns the two research cards it produced.

## The engine

- About 5,500 lines of Nim, GPLv3, one author. Search is 800 lines.
- Official CCRL entries are for 1.5: **3258** at 40/15 (rank 186) and
  **3341** at blitz (rank 153). The README's 3550 for 1.6 is the author's
  estimate from a 400-game match against Elixir 3.0 and is unconfirmed.
- Evaluation: one embedded 768×1024 perspective network, horizontal king
  mirroring (file e–h flips), squared clipped ReLU, one output, no output
  buckets, no king buckets beyond the mirror. Lazy per-ply refresh flag when
  the king crosses the mirror boundary; no refresh cache. Hand-crafted
  evaluation removed at 1.5. SIMD through C intrinsics imported by name
  (AVX2, AVX-512, NEON), scalar fallback kept.
- Search: PVS, aspiration with a fail-high depth reduction, 3-entry clustered
  TT with generation aging and prefetch, NMP `R = 2 + depth/4` with
  verification above depth 14, RFP with a continuous clamped improvement
  term, futility, LMP `3 + d²`, quiet and noisy SEE pruning, singular with
  double, negative and multicut (lerp), IIR at no TT move, check extension,
  threat-indexed main history `[stm][from][to][fromAttacked][toAttacked]`,
  two continuation histories, pawn and non-pawn correction history, two
  killers, MVV-LVA captures, node-fraction plus best-move-stability time
  management, lazy SMP with LMR jitter and vote-based thread selection.
- Absent: capture history, ProbCut, razoring, history-based pruning, TT-PV
  tracking, continuation and minor-piece correction.

## Nim against Rust

Nim compiles to C, so the binary goes through GCC or Clang with the same
optimiser, PEXT, prefetch and intrinsics that Rust reaches through
`std::arch`; speed parity is realistic and the intrinsic bindings are a few
lines each. The costs are visible in this code base: thread-local globals
for the NNUE state and killers, manual shared allocation for the history
tables and the TT, and a small ecosystem, so every tool is home-grown. Rust's
borrow checker over a 14,000-line search and cargo's test matrix are worth
more to Rarog than any Nim advantage. No language decision changes.

## Where the strength is

A search simpler than Rarog's accepted core, carrying a plain 768×1024 net
trained on the author's own data, lands in the CCRL top 200 by official
numbers. The 1.5 to 1.6 progression (+345 ± 22 Elo in the author's 1,000
games) coincides with the 1024-wide net. The rating is the network. This is
consistent with PLAN's budget: the search deficit is being closed in Phase
B, and the evaluation gap is where the rating lives.

## What Rarog already has

Threat-indexed quiet history (`update_quiet` takes `threats().all`),
cut-node reductions, the multicut lerp (`CoreSingMulticutLerp`), a continuous
improving term in RFP, four correction histories against Gyatso's two,
node-fraction and instability time management, thread voting and LMR
jitter. Where the two differ, Rarog's form is the more elaborate one.

## Findings carried into PLAN

1. **TT-hit history bonus** (search card, B.5.1): on a TT cutoff with a
   lower bound and a quiet TT move, Gyatso gives that move half the depth
   bonus. Rarog's TT-cutoff path writes no history. Stockfish's form is the
   donor reference.
2. **Draw-score randomisation** (search card, B.5.1): repetition and
   rule-50 returns are `nodes mod 5 − 2` instead of a fixed draw score.
   Rarog returns a fixed score. Stockfish's form is `1 − (nodes & 2)`.
3. **Calibration point for F.0**: a single-bucket 768×1024 mirrored net on
   a plain search reaches CCRL 3258/3341 official. F.0's first architecture
   (768×N with output buckets) is at least that.
4. **Datagen reference for F.0 and F.2**: node-limited self-play
   (soft and hard node caps), an opening book sampled by inverse use-count,
   viriformat output. A compact reference for F.0's data-format contract.
