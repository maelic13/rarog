# Ablation harness: design, and why it is not a bisection over the oracle

## Stockfish already published the decomposition

Every top-level step in the oracle's `search.cpp` carries its own Elo value,
written by Stockfish's authors from their own testing. 22 annotated sites:

| step | mechanism | their figure |
|---|---|---:|
| 13 | Pruning at shallow depth | **~200** |
| 16 | Late move reductions | **~200** |
| 14 | Extensions (singular ~70, check ~2) | ~75 |
| 8 | Futility pruning, child node | ~50 |
| 9 | Null move + verification | ~40 |
| 10 | ProbCut | ~10 |
| 7 | Razoring | ~1 |
| 11 | Internal iterative deepening | ~1 |

Sub-terms inside LMR are annotated too: history-based reduction ~30, cut-node
~10, stat-score ~10, on-PV ~10, tt-capture ~5, opponent move count ~5,
singular-extended ttMove ~3.

**So a bisection over the oracle's mechanisms would spend a day rediscovering
numbers that are already in the source.** Bisection is the right instrument
when there is no prior. Here there is one, for free, and it should be used and
then checked rather than re-derived.

## The finding that reframes the work

**Rarog already has every one of these mechanisms.** Razoring, futility, NMP
with verification, ProbCut, IIR, shallow-depth pruning, singular extensions,
LMR -- all present. So the ~196 Elo deficit is NOT a list of missing features.
It is that Rarog's version of each mechanism captures less of that mechanism's
available Elo.

That cannot be measured by ablating the oracle alone.

## The right experiment: PAIRED ablation

For each mechanism M, measure the Elo each engine loses when M is removed:

    delta_SF(M)     = Elo(oracle)  - Elo(oracle without M)
    delta_Rarog(M)  = Elo(Rarog)   - Elo(Rarog  without M)

**`delta_SF(M) - delta_Rarog(M)` is the unrealised Elo in Rarog's version of
M**, and it localises the deficit directly. If removing LMR costs the oracle
200 and costs Rarog 120, Rarog's LMR is leaving ~80 on the table and that is
where the work belongs. If the two deltas match, Rarog's version is already
doing that mechanism's job and no amount of Stockfish-shaped rework will pay.

This is the measurement the phase has been missing. Every previous instrument
-- counter rates, fixed-depth agreement, fixed-node agreement -- reported
something correlated with strength at best. This reports Elo.

## Why the economics work

Ablation effects are 40-200 Elo, and **large effects are cheap to measure**.
RAR-S70 gave +/-1.85 Elo over 56,928 games, so +/-8 Elo needs about 3,000
games -- roughly 30 minutes. Eight mechanisms on two engines is ~16 arms, well
inside a day.

The phase's structural mistake was measuring small effects expensively. This
inverts it.

## Where bisection IS the right tool

Two places, and the harness is built for it: the bitmask makes any subset one
number, so a half is a single run with no rebuild.

1. **The sub-terms.** LMR's seven annotated adjustments sum to ~73 Elo and
   Rarog implements them unevenly. Bisecting those is cheaper than eight
   separate arms.
2. **The additivity check, which bisection silently assumes.** Run a half AND
   its complement. If effects were additive, `delta(A) + delta(B)` would equal
   `delta(A and B together)`. The gap between them measures interaction
   directly, and it is the number that says whether bisection is even valid
   here. Given that Rarog gained +30.75 Elo REMOVING its check extension --
   the opposite of the oracle -- interaction is expected to be large, and it
   is better measured than assumed.

## Caveats, stated before any number is produced

- Stockfish's annotations are approximate, from their hardware, their time
  control and their evaluation. Use them for ORDERING, and measure magnitudes
  in the hybrid where Rarog's HCE is what is being searched.
- Every ablation delta is a MARGINAL value inside a co-adapted stack. It says
  what the mechanism is worth given everything else that engine does, not what
  it would be worth in isolation.
- Removing a pruning mechanism at a fixed clock makes an engine search a much
  larger tree. That is the intended effect, but it means each ablated arm is
  weak in a way that may compress differences. Fixed-node arms would remove
  the clock but also remove the thing being measured, so the clock stays.

## The harness

Instrument: branch `hybrid-ablate`, one UCI option `AblationMask`, one bit per
step, verified live bit by bit before use. It never merges anywhere -- no C++
enters Rarog, which reimplements in Rust from the principle.

Runner: `tools/sprt.ps1` already takes `-OptionsA` / `-OptionsB` and a
`-Mode fixed` fixed-size match, so no new runner was needed.

Rarog's half is not built yet: it needs a matching ablation switch per
mechanism so `delta_Rarog(M)` can be measured the same way.
