# Move-list delivery probe — RAR-M44

Research record for leaf **4.11b.19**. Zero games; one within-Rarog board
microbenchmark A/B plus assembly inspection. No engine source was changed on
`dev`; the probe diff is archived, not committed.

## Question

RAR-M20/RAR-M43 measured Basilisk 44–46% faster than Rarog on "legal moves"
and 21–26% on "legal captures", on a generator that is a line-for-line port of
Basilisk's `gen_legal_impl` (`D:/code/basilisk/src/board.cpp:1007`). An
identical algorithm cannot lose 46% to itself; the gap had to be constant
factors, and one of them turned out to be the harness.

## Evidence 1 — the emitted assembly

Built the production bench with the RAR-M20 recipe flags
(`RUSTFLAGS='-C target-cpu=native --cfg rarog_pext'`, no features,
`cargo rustc --release --bench board --no-default-features -- --emit=asm`,
fat LTO). In the LTO'd output:

- `generate_legal_movelist` builds the list in a stack local and its **normal
  return path** ends with `movl $520, %r8d; callq memcpy` — a copy of the
  entire 256-slot `MoveList` into the caller's out-pointer. RVO did not apply.
- `generate_captures` ends the same way (`$520` bytes).
- 4.11b.8 had already seen the same 520-byte `memcpy` in
  `generate_captures_pinned` (tuple return) and dropped the question.
- Also out of line inside `generate_legal_movelist`: `push_pawn_move_flags`
  (4 call sites, despite `#[inline]`), `is_attacked_with_occ` (per king target),
  `compute_pinned` (1); two `panic_bounds_check` sites; seven `LazyLock`
  state checks (all on cold branches — cheap, but present).

Basilisk's harness (`tests/board_performance.cpp:115`) passes a reused
`MoveList&` precisely because its author found that returning by value "put a
1 KB aggregate copy inside the timed region ... and measured the copy as if it
were move generation". Rarog's `benches/board.rs` returns by value. The two
"legal moves" columns therefore never measured the same work.

## Evidence 2 — the probe

Two bench executables from the same tree, same flags, differing only in the
probe diff (`tools/results/board-copy-probe-20260909/probe.diff`, 192 lines):
`MoveList::clear()`, `generate_legal_into(&Board, &mut MoveList)`,
`generate_captures_into(&mut Board, &mut MoveList)`, and the four affected
workloads rewritten to reuse caller-owned lists. **Threshold SEE and perft(4)
were deliberately left untouched as controls.** Host busy 0.8–6.4% before the
run; runner pinned to affinity mask 4; order base, variant, variant, base,
base, variant; median of three runs per arm.

| Workload | base (M ops/s) | variant | change | spread base / variant |
|---|---:|---:|---:|---|
| legal moves | 443.54 | 492.99 | **+11.15%** | 1.55% / 0.38% |
| legal captures | 95.51 | 134.21 | **+40.52%** | 3.01% / 0.59% |
| make/unmake | 51.99 | 52.60 | +1.17% | 0.28% / 0.40% |
| threshold SEE (control) | 45.52 | 45.54 | +0.05% | 1.66% / 0.22% |
| perft(4) (control) | 290.25 | 288.24 | −0.69% | 0.59% / 0.46% |
| two-ply simulation | 384.29 | 402.06 | +4.62% | 0.63% / 0.28% |

Both controls sit inside their spreads, so the session resolves the effect.
Base "legal moves" reproduces RAR-M43's same-day head figure (444.99) within
0.3%, which is why the RAR-M43 Basilisk figures can be read against this
table directionally: with the copy removed Rarog's capture generation
(134.2) exceeds Basilisk's 120.8, and the legal-moves gap shrinks from
46.2% to about 32%. That cross-arm reading is between sessions and is
directional only; 4.11b.19(d) re-measures all four arms in one session.

## What this changes in the 4.11b record

- RAR-M43's statement that generation "was untouched, so the unchanged
  generation gap is the expected outcome" is true of the code and false of the
  measurement: roughly a quarter of the legal-moves gap and most of the
  capture gap were the harness.
- The production search pays the same copy: `MovePicker::staged` calls
  `generate_legal_captures_pinned()` and the `GenerateQuiets` stage calls
  `generate_legal_quiets_pinned()`, both by value, once or twice per node;
  `generate_legal_movelist()` is used at the root, at in-check/excluded
  nodes and in in-check quiescence. RAR-M36 puts generation at 6.556% of
  search time; the copy is a bounded fraction of that, so the whole-search
  prediction is small and is registered in RAR-M44 before it is measured.
- RAR-M43's Elo arithmetic priced only generation and make/unmake. It
  excluded SEE (5.239%, where Basilisk leads ~30% at matched values per
  RAR-M29) and check queries (5.179%, never compared). Including them at a
  similar ratio doubles the "whole board parity" figure to roughly +5% NPS,
  about 10 Elo at the STC constant — still an order of magnitude below the
  measured search deficit, so 4.11b's prioritisation stands.

## Reproduce

```
python tools/results/board-copy-probe-20260909/ab.py
```

Directory contents: `probe.diff`, `ab.py`, `ab.log`, `ab_result.json`,
`board-base.exe`, `board-variant.exe`, `binaries.sha256`, build logs,
`README.txt`. Base SHA-256 `a64057a1…`, variant `ddb78137…`. Source `c1a7713`
plus `probe.diff`. Ignored directory; not in Git.

## Implementation and the registered production measurement (2026-09-09)

Committed as `021dc98` (engine) and `55e228a` (harness); recorded in PLAN
4.11b.19 and RAR-M44. One deviation from the registered caller list, reported
rather than taken silently: **ProbCut's capture generation** reached the same
copy through `Board::generate_legal_captures` and was converted with the rest.
The assembly scan is what found it — after the registered sites were done it
was the only 520-byte copy left in the binary.

Mechanical proof: the fat-LTO binary has **zero** `movl $520` + `callq memcpy`
sites, against **four** before (`movegen::generate_captures`,
`movegen::generate_legal_movelist`, `MovePicker::staged`, `MovePicker::next`).
The same scan still reports the copy inside the three surviving by-value
wrappers, so a clean line is a measurement and not an absent pattern. `bench
13` reproduces **7,601,220 / EBF 2.474** on magic and PEXT.

**Registered pooled-PGO result, one run, maintainer, idle host:**

| Arm | pooled median n/s |
|---|---:|
| base (`f10b999`, three PGO builds) | 3,142,298 |
| cand (head, three PGO builds) | 3,220,173 |

**+2.48% whole-search NPS, 95% bootstrap [+2.29%, +2.65%]**; best-of +2.81%.
Null pair `cand-1` vs `cand-2`, same revision: **−0.21% [−0.57%, +0.23%]**.
All six binaries reproduce the fingerprint and all six hashes differ.

**BANKED.** The lower bound is more than four times the registered +0.5% floor.

**Calibration, written after exposure.** The frozen band was +0.5% to +1.5%;
the measurement is +2.48%. The miss is in **magnitude only** — sign, floor and
the ceiling argument above (RAR-M36's 6.556% generation share) all held, and
2.48% sits well inside that ceiling. What failed is the sentence in this
document that reads "the copy is a bounded fraction of that": it was about
**38%** of the generation share. Two candidate reasons, neither established
here: the copy was not confined to generation — ProbCut and quiescence paid it
at the caller's return slot, outside the symbol a share profile charges
generation to — and 520 bytes of store traffic per call costs more than its
instruction count implies. No Elo is claimed; behaviour is unchanged and
RAR-E15's verdict stands.

## (c) and (d), 2026-09-09

**(c) constant-factor screen: 2 of 4 candidates promoted on the bench, then
the bundle REJECTED in search and reverted.** Force-inlining three helpers
(+4.68% legal moves) and const-generic colour (+7.38%) cleared the registered
+3% gate; an unchecked `MoveList::push` (-9.57%) and hoisting six of seven
`LazyLock` state checks (-4.25%) did not, and were reverted unscreened
further. The bundled pooled-PGO run then measured **-0.55% whole-search NPS,
95% [-0.76%, -0.30%]**, with a null pair at +0.06% [-0.46%, +0.44%], against
a registered +0.5% floor and a frozen prediction band of +0.8% to +2.2%.
Both commits were reverted, as the registration required.

**The prediction missed in SIGN.** The two candidates were worth +12.7% on the
legal-moves and two-ply bench columns and NEGATIVE half a percent of search.
The stated risk written down before the run -- candidate 2 roughly doubling
the generator, whose I-cache cost a microbenchmark running one hot generator
cannot see -- is the leading explanation and is NOT established; the pair was
gated as one bundle, so neither candidate is individually credited or blamed.

**(d) re-measured the four arms twice**: once on the (c) head, and again on
the (b) head after the revert. Current record and the superseded (c)-head
table: `analysis/board_comparison_411b19_2026-09-09.md`. The legal-moves gap
to Basilisk is **33.2%**, down from RAR-M43's 46.2%, and capture generation is
**5.8% ahead** of Basilisk. This document's own directional prediction for (b)
-- "about 32%" and "about 11% ahead" -- scores as exact on the first and
overstated by about half on the second.

**What this leaf establishes about its own instrument:** (b) removed work and
+11% legal moves converted to +2.48% of search; (c) moved work around and
+12.7% legal moves converted to -0.55%. A board microbenchmark column is not a
proxy for search speed on this codebase, and its SIGN does not always survive.
