# D.16.3 — the quiet-scoring history loads prefetched ahead: registration (2026-11-24)

Registered before the item is built or read. The result is appended below;
nothing above *Result* changes afterwards.

## The item

This is D.16.3.1's one item (`analysis/d1631_node_profile_2026-11-23.md`,
*Items*, row 1). In the node's quiet-scoring loop
(`append_compact_quiets`), each move's score loads six history entries:
the threat-indexed quiet entry, the pawn-structure entry and four
continuation entries. 3.5% of the head's samples wait on those loads. The
item prefetches the six entries of the move **four places ahead** before
the current move is scored. The first four moves' entries are prefetched
before the loop starts. The distance is fixed here, before any build, and
is not tuned on the read.

- **Mechanism:** load latency the out-of-order window does not hide. The
  continuation table is 4.7 MB and the pawn table 0.8 MB, both larger than
  L2. The quiet table is 64 KB, but it is prefetched as well, since it
  shares the address arithmetic.
- **Interactions:** none with play. No value changes, so the tree is the
  same. A prefetch competes only for cache and fill buffers, and could
  evict a line the node is about to use. Code layout moves under PGO.
- **Invariants:**
  - The `bench 13` fingerprint stays exactly **3,694,298 / EBF 2.324**.
  - AGENTS' per-node rule holds: no allocation, no clone, no buffer beyond
    the move list, and no `memcpy` or `memset` in the picker's or the
    node's assembly.
  - No new `unsafe` site (the floor is 17): the hint goes through
    `tt.rs`'s existing `prefetch_ptr`, which is made visible to the
    crate.
  - fmt, clippy over all features at zero warnings, and the debug and
    release tests all pass.
- **Not in the item:** the root picker's quiet loop, which runs once per
  iteration; the noisy and evasion scoring; and any change of order or
  value.

## Read (frozen)

The base pool is D.16.1's `analysis/artifacts/d161-2026-11-23/pool-a`.
It holds four pext PGO builds of `415ec4f`, whose engine source is the
head's: every commit since changes only documentation and tools. The
candidate pool is four pext PGO builds of the item's commit, from a clean
tree, at the same fingerprint. The toolchain is the same, rustc 1.100.0.

**Deciding read: the bench read** (D.16.2 kept it), by PROCESS's two
steps, against the base pool:

- Step 1, two cycles: accepted at +0.9% or more; `NO_CHANGE` at +0.1% or
  less.
- In between, step 2, four more cycles: accepted at +0.5% or more with the
  lower bound above 0.
- **Otherwise the item is reverted and D.16.3 closes `NO_CHANGE`.** That
  covers D.16.3's frozen clause (an interval that does not exclude zero
  reverts it). It also covers a step-2 result that is in between with its
  interval above zero, because the item adds code: D.7.8.12's clause keeps
  a between result only when it removes code.
- An accept below +1% is recorded as possibly code layout, by PROCESS's
  rule.

**Beside it, not deciding: the game-position read, step 1**, on the same
pools. Its use: a second memory change for D.16.2's open observation that,
on such a change, the bench read's step 1 did not exclude zero where the
game read's did. The two are compared by D.16.2's band,
`|Dg − Db| ≤ sqrt(hb² + hg²)`, using the bench read's step 1. It does not
decide this item.

```
python tools/nps_read.py --base analysis/artifacts/d161-2026-11-23/pool-a --cand analysis/artifacts/d163-2026-11-24/pool --out analysis/artifacts/d163-2026-11-24/bench --label "D.16.3 item 1 bench: history prefetch against the head"
python tools/nps_read.py --extend analysis/artifacts/d163-2026-11-24/bench   # only when step 1 is between
python tools/nps_read.py --base analysis/artifacts/d161-2026-11-23/pool-a --cand analysis/artifacts/d163-2026-11-24/pool --positions tools/nps_positions_v1.jsonl --out analysis/artifacts/d163-2026-11-24/games --label "D.16.3 item 1 games: history prefetch against the head"
```

## Prediction (frozen before the build)

- **Bench +0.3%** (80% band −0.2 to +1.0). The loop's iterations are
  independent, so out-of-order execution already overlaps much of the
  latency. D.7.8.12's analogous second prefetch read +0.14%.
- **Games +0.4%** (80% band −0.2 to +1.2).
- P(accepted) 0.25.

**Band, inferred and not measured:** at RAR-M92's 1.67 Elo for 1% of node
rate, the predicted +0.3% is about +0.5 Elo. The ceiling, 3.6% of rate, is
about +6 Elo.

## Result (2026-11-24): `NO_CHANGE`, reverted

**Build.** `90c4460`: `prefetch_ptr` is now visible to the crate (the same
`unsafe` site, so no new one). The history tables hand out the address of
the quiet, pawn and continuation entries. `append_compact_quiets` asks for
the first four moves' lines before its loop, and for the line of the move
four ahead at each step.

**Checks.**

- `bench 13` was 3,694,298 / EBF 2.324, exact.
- fmt passed, and clippy over all features gave zero warnings.
- Debug tests: 555 passed. Release tests: 557 passed.
- The release library's assembly (`asm_scan.py`) has no `memcpy` or
  `memset` in the node, quiescence, `CompactPicker::next`, make or
  `Evaluator::evaluate`. The two hits it found are `Evaluator`'s `Default`
  constructor, which is not per node.
- `CompactPicker::next` gained six `prefetcht0` instructions, so the item
  was active.

**Reads.**

- Candidate pool: four pext PGO builds of `90c4460`, clean, all at
  3,694,298.
- Base pool: `analysis/artifacts/d161-2026-11-23/pool-a`.
- The host was idle (0.0 to 0.6% CPU), and no cycle was disturbed.
- In the games read the two arms searched the same 200 trees.

| Read | Delta | 95% interval | Verdict |
|---|---|---|---|
| Bench, step 1 (deciding) | **−4.74%** | −6.49% to −2.99% | `NO_CHANGE` by step 1 |
| Games, step 1 (beside) | **−3.67%** | −3.89% to −3.45% | — |

**By the frozen rule the item is reverted** (`22b4310`; the source is
byte-identical to `90c4460^`, and `bench 13` is 3,694,298 again), **and
D.16.3 closes `NO_CHANGE`.** The rest of the gap is the tree, its
mechanisms and the evaluation's form, as D.16.3.1's card put it.

**The second memory change, for D.16.2's observation.** The gap between
the two reads is +1.07 against D.16.2's band of 1.76, so they agree. The
games read's interval is again the narrower (±0.22 against ±1.75). Here
the width is the base pool's own spread on the bench. Its four builds read
2,577,458 to 2,644,517 nodes a second, 2.6% apart. On game positions the
same builds read 2,353,241 to 2,362,278, 0.4% apart. D.16.1 had read this
pool only on game positions. Two memory changes now show the bench read
the wider of the two. That is an observation, not a decision.

**Calibration.** The prediction was +0.3% on the bench (80% band −0.2 to
+1.0) and +0.4% on games (−0.2 to +1.2). The reads were −4.74% and −3.67%:
a miss in sign and magnitude. The assumption that failed was that a
prefetch costs only cache. The item made each quiet move pay for its own
address work again, a second `moving_piece` lookup and six hints, and
filled L1 with lines four moves early. The scores waiting on the loads did
not get faster. Nothing here measures which of these cost the 3.7%. The
read is a refutation of the item, not an explanation of the loss. As with
D.7.8.12's second prefetch, a profile's share of stalled instructions did
not show that the loads were on the critical path. The cause is not
investigated further: the rule closes the step and no use is named for the
explanation.

**Band, inferred:** none to price. The pass leaves the node rate as it was.
