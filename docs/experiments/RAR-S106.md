# RAR-S106 — D.5.5: `Hash` at any size, the repair's speed and harm reads

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-06 before any game and any speed
  read. The builds and the speed read need an idle host; the match is the
  maintainer's command.
- **Kind:** AGENTS' repair case 2, with no Elo claim: a correction that moves
  the fingerprint, expected near zero, where a small loss is acceptable for
  correctness. D.5.5 replaced its notice with this repair by maintainer
  decision 2026-11-06, moving the item up from G.2.
- **Baseline / candidate:** `9b8538d` (bench 14,749,043 / EBF 2.564) and
  `1450340` (bench 15,625,852 / EBF 2.563). They differ in `src/tt.rs` and
  two test files only. Each arm builds from a clean worktree at its commit;
  the binaries' hashes are appended here once built, before any game.
- **Research question:** does exact sizing, with a multiply-high index and
  the tag in the key's low 16 bits, cost speed or strength at the size every
  gate uses?
- **Hypothesis / mechanism:** none for strength. `Hash` was floored to a power
  of two (1000 gave 512 MiB). Reckless and Stockfish 19 both size exactly and
  index by the product's high half; Rarog now does the same. At a power-of-two
  `Hash` the table keeps its size and only the slots move.
- **Competing hypotheses:**
  - The low-bit tag weakens verification. Zero-game read against it: six valid
    tag and index choices bench 11.9M to 15.6M around the old 14.75M, 18
    positions up and 17 down with a median ratio of 1.000, while an index
    overlapping the tag reads 53.0M at EBF 2.80. A broken tag is that visible,
    and this change is not.
  - The extra multiply in `probe`, `store` and `prefetch` costs speed. Both
    donors pay it; this read measures it.
- **Interacting mechanisms / consumers:** the table's replacement and
  `hashfull` (unchanged), the shared table's `key16 ^ fold16` verification (the
  same 16 bits of strength), `prefetch` and `key_after_hint` (the same index),
  and every signal that reads a table hit, through collisions only.
- **Exposure:** every table access in every game, so all of them, by
  construction. Gates run at a power-of-two `Hash`, so the games read the
  remapping alone; the larger table a size such as 1000 now gets cannot be read
  from them, and is read by allocation: `Hash 1000` holds 1,042 MiB against 554
  at one thread, 1,163 against 675 at four.

## The reads (frozen)

- **`nps`:** four pext PGO builds an arm, `tools/nps_read.py --no-regression`,
  PROCESS's method. Step 1 passes at −0.1% or above and fails at −0.9% or below;
  in between, step 2. The arms search different trees (+5.9% nodes), so this read
  also carries that tree's mix, and it cannot be separated here.
- **`stc`:** Colosseum `match`, 2,000 games at `3+0.03`, Hash 64, one thread,
  the wrapper's slot count, the UHO book in random order, no adjudication, seed
  20261201, `tools/results/rar-s106-stc`.
- **Commands:** `run_s106.ps1 -Step build|nps|stc` in
  `analysis/artifacts/d55-hash-2026-11-06/`, in that order.
- **Harm rule** (AGENTS' case 2, from the manifest's nElo): below **−13 nElo**
  (2,000 games, standard error about 7.8). This catches a true −10 with
  probability 0.36 and −20 with 0.82, so "kept" means no harm beyond roughly −20
  nElo.

## Dispositions

- **Neither read fails:** D.5.5 closes, the fingerprint stands, and the
  changelog gets its entry.
- **The harm rule fires:** the repair leaves `d5` by the gate, the defect stays
  open in D.5.5, and a rework is a new registration.
- **The speed read fails:** the read goes to the maintainer with the game read
  beside it. The repair is a correctness fix for users, so a small cost is
  theirs to weigh; neither outcome is decided here in advance.

## PRE-REGISTERED PREDICTION (frozen before exposure)

- **`stc`:** 0 ± 15 nElo. P(no harm) 0.95, which is the rule's own false-alarm
  rate at a true zero.
- **`nps`:** −0.2% ± 0.4%, P(pass) 0.75. One multiply replaces one mask, off the
  memory latency that dominates a probe.
- **Most likely failure mode:** the speed read landing between its bounds on
  code placement or the trees' mix, leaving step 2 unresolved rather than failed.
