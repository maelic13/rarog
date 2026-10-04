# The answer harness cannot rank candidates. Calibrated, 2026-08-21.

## What was tested

Two changes with KNOWN game results were run against the oracle at a fixed
node budget, together with their true baselines:

| arm | binary | bench | known result |
|---|---|---|---|
| pre-4.7 base | `rarog-47prebundle` (rebuilt from `aaa715a`) | 6,519,711 | baseline |
| 4.7c | `rarog-47c-only` | 6,922,439 | **+15.56 Elo** (RAR-S58) |
| pre-S70 head | `rarog-46base` | 7,467,143 | baseline |
| RAR-S70 | `rarog-46root` | 6,977,070 | **+2.33 Elo** |

`go nodes 300000`, 50 positions, evaluation held constant.

## Result

| arm | agreement | mean depth | vs its base | med \|dcp\| |
|---|---:|---:|---:|---:|
| pre-4.7 base | 80.0% | 24.34 | -- | 31 |
| **4.7c (+15.56)** | 70.0% | 24.60 | **+0.26** | 25 |
| pre-S70 head | 70.0% | 24.68 | -- | 26 |
| **RAR-S70 (+2.33)** | 80.0% | 25.10 | **+0.42** | 36 |

Oracle mean depth 28.86, so Rarog is ~4.2 plies short at equal nodes. That gap
is real and is the axis fixed-depth comparison discards.

**But no metric orders the two known results.** Agreement moves the WRONG way
on the larger gain (80 -> 70%) and the right way on the smaller (70 -> 80%).
Mean depth gives the +15.56 change a SMALLER improvement (+0.26 ply) than the
+2.33 change (+0.42). Median |dcp| happens to order them correctly, which on
two points is a coin flip.

## The honest reading: this is resolution, not anti-correlation

On 50 positions a 10-point agreement swing is five positions, about 1.4 sigma.
All four arms sit inside each other's noise on every metric. The correct
statement is not "agreement is anti-correlated with Elo" -- it is **the suite
cannot resolve a sevenfold difference in Elo**, so it cannot rank anything.

That applies to the fixed-DEPTH harness too, and retroactively to how RAR-S70
was selected. The relief was chosen because agreement moved 66 -> 78% across
three depths, monotone in the parameter. It then measured +2.33 Elo -- a real
gain, so the selection was not harmful, but the evidence that picked it is now
known to be unable to distinguish that gain from a much larger one.

## What this does and does not license

- **Do not use oracle agreement to rank candidates or to decide what earns a
  gate.** Two calibration points, and it fails on both counts.
- **Do keep the harness for MECHANISM questions**, where it is sound: it
  correctly showed that the reduction formula cannot see the root, and that
  `LmrMinReducedDepth` removes 18.4% of the tree. Those are structural facts,
  not rankings.
- **The 4.2-ply deficit at equal nodes is the one ranking-relevant number here**
  and it is large. It says Rarog's remaining gap to the oracle is mostly the
  cost of reaching depth, which is what the fixed-depth instrument was blind to.
- **The binding constraint is suite size, and the fix is cheap.** Fifty
  positions is far too few. Enlarging to ~1,000 cuts the noise roughly
  fourfold and costs minutes per arm, against ten hours for a gate. No
  zero-game ranking should be trusted until the enlarged suite reproduces the
  4.7c / RAR-S70 ordering.

## An evidence-integrity problem found on the way

`tools/test_engines/rarog-47base-pext-pgo.exe` is cited by RAR-S58 at bench
**6,519,711** and by RAR-S67 at **7,467,143**. Two different experiments reused
one filename, so RAR-S58's baseline binary no longer exists. It was recoverable
only because the row also records the SHA, and `aaa715a` rebuilds to 6,519,711
exactly -- which is precisely the reason the ledger rule demands a recipe and a
fingerprint rather than a path. **Experiment binaries must never reuse a name.**
