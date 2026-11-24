# D.16.2 — the bench read against play: registration (2026-11-23)

Registered before any reading of these pools. The result is appended
below, and nothing above *Result* changes afterwards.

## The use

This step decides which instrument prices the coming speed work: the bench
read (`bench 13 3`, PROCESS) or the game-position read (D.16.1,
`--positions tools/nps_positions_v1.jsonl`). The first user is D.16.3's
one item, which prefetches the quiet-scoring history loads. That is a
memory change, so the memory change below decides how that item is read.
Trinity's record is why the question is open: a picker change there read
+5.6% on its bench and 18% faster on game positions.

## Arms

Two accepted, behaviour-neutral speed changes of different kinds on the
compact kernel. Each is read against the build it was accepted against,
`bench 13` 3,320,355 / EBF 2.317 on every arm:

| Change | Kind | Base | Candidate | Its accepted bench read |
|---|---|---|---|---|
| A, D.7.8.12 item 1 | memory: the child's table cluster prefetched from the hint's key before the move is made | `b88700a` | `1ff6522` | +1.09% (+0.52 to +1.67), step 1 |
| B, D.7.8.9 | node work: the non-pawn keys kept with their kings, the reverse-futility and null-move gates asked only where they can act, the root list filled in place | `42e3073` | `b88700a` | +0.95% (+0.37 to +1.54), step 2 |

A's accepted base was `b92cd4b`, a documentation commit whose engine inputs
equal `b88700a`'s: `git diff b88700a b92cd4b`, outside `docs/`,
`analysis/` and Markdown, is empty. So one pool of `b88700a` serves as A's
base and B's candidate.

The original pools' executables were cleaned out; only their manifests are
left. Three pools are therefore rebuilt, each from a detached worktree of
its commit with that commit's own `tools/nps_build_pool.ps1`: four pext
PGO builds, rustc 1.100.0, the fingerprint verified on every copy. The
driver is `analysis/artifacts/d162-2026-11-23/build_pools.ps1`; the
pools' hashes are under *Pools*.

## Reads

Each change is read twice on the same two pools: first the bench read, then
the game-position read. Each read is PROCESS's step 1, two cycles, under
its disturbance rule: when an arm's two cycles disagree by more than 1%, the
step is repeated. No step 2 is run. PROCESS puts the builds' limit near
±0.25% at any number of cycles and step 1 at about ±0.4%, so a pair of
step-1 reads resolves a gap of about 0.6 points between a change's two
deltas. That is about half of each change. A smaller gap is inside what
either read already tolerates, so it would not change which instrument
prices a change of this size.

```
python tools/nps_read.py --base analysis/artifacts/d162-2026-11-23/pb88700a --cand analysis/artifacts/d162-2026-11-23/p1ff6522 --out analysis/artifacts/d162-2026-11-23/a-bench --label "D.16.2 A bench: 1ff6522 against b88700a"
python tools/nps_read.py --base analysis/artifacts/d162-2026-11-23/pb88700a --cand analysis/artifacts/d162-2026-11-23/p1ff6522 --positions tools/nps_positions_v1.jsonl --out analysis/artifacts/d162-2026-11-23/a-games --label "D.16.2 A games: 1ff6522 against b88700a"
python tools/nps_read.py --base analysis/artifacts/d162-2026-11-23/p42e3073 --cand analysis/artifacts/d162-2026-11-23/pb88700a --out analysis/artifacts/d162-2026-11-23/b-bench --label "D.16.2 B bench: b88700a against 42e3073"
python tools/nps_read.py --base analysis/artifacts/d162-2026-11-23/p42e3073 --cand analysis/artifacts/d162-2026-11-23/pb88700a --positions tools/nps_positions_v1.jsonl --out analysis/artifacts/d162-2026-11-23/b-games --label "D.16.2 B games: b88700a against 42e3073"
```

## Rule (frozen)

For each change, let the bench delta be `Db` with 95% half-width `hb` and the
game-position delta be `Dg` with half-width `hg`, both as `nps_read.py`
reports them. PLAN's "agree within their combined intervals" is taken as:
the two deltas agree when `|Dg − Db| ≤ sqrt(hb² + hg²)`. That is the 95%
interval of the difference, treating the two reads as independent. They
share builds, so this is approximate; any shared build luck makes it
cautious. Whether the two intervals merely overlap, the looser reading, is
also reported, but it does not decide.

- **Both changes agree:** the bench read stays the speed read, and D.16
  closes on D.16.2. D.16.3 stays open by the maintainer's decision, and its
  item is read by the bench read.
- **A change disagrees:** the game-position read becomes the speed read for
  that change's kind. Change A's kind is memory, so D.16.3's item is then
  read by the game-position read. Change B's kind is per-node work in the
  move loop.
- **Different game trees:** if a change's two arms search different game
  trees (the read's tree warning), that change is not behaviour-neutral on
  play positions. Its game delta is then not a speed delta, the change does
  not count toward the rule, and this is recorded as a finding.

## Prediction (frozen before any reading)

- A: bench +1.1% (80% band +0.5 to +1.7); games +1.3% (+0.5 to +2.2).
  Both reads use Hash 64, which is larger than the 5950X's L3, and the
  bench table is warmer, being shared over forty positions. A prefetch that
  hides a table miss may therefore pay slightly more where more probes miss.
  P(agree) 0.75.
- B: bench +0.9% (+0.3 to +1.5); games +0.9% (+0.2 to +1.6). The gates'
  saving scales with how often an earlier gate ends the node, which the two
  position mixes may set a little differently. P(agree) 0.8.
- Both agree, so D.16 closes with the bench read: P 0.6.

Falsifier of the bench read: a change whose game delta falls outside the
rule's band. The step stops at these four reads: a disagreement is not
chased with more reads here; it is handed to D.16.3's pricing.

## Pools

Built 2026-11-23, 21:24 to 21:29. Four pext PGO builds a pool, rustc 1.100.0, every
copy at 3,320,355; the full SHA-256 values are in each pool's
`manifest-pext.txt` under `analysis/artifacts/d162-2026-11-23/`, checked
against the files. The worktrees were removed after the build.

| Pool | Source | SHA-256, first 16 hex digits |
|---|---|---|
| `p42e3073` | `42e3073`, clean | 68F98C989C26C66C, F468DA79F6C21235, F36CA11905E0C65D, 8DB90C0F46824882 |
| `pb88700a` | `b88700a`, clean | DDD115F5BC0BDC03, 85C2200EDB9BB4C2, 5CB1B2BC970DAF83, 67F085397DB773F6 |
| `p1ff6522` | `1ff6522`, clean | 608EDFD71C79B751, B97D5E9E031D1716, FD844E32DF10832C, 98DBDCDC3892EB39 |

Untimed smoke (`smoke.py`, one build of each pool, five positions of the
set): the older binaries follow the game-position dialogue, and each
change's two arms search the same trees there. Rates were not read.

## Result (2026-11-24)

The four reads ended between 07:39 and 07:55 on an idle host. The CPU before each
cycle was 0.0% to 0.5%, no cycle was disturbed, and the bench fingerprint
was 3,320,355 on every arm. In each games read the two arms searched the
same 200 trees (best moves and last counts). Raw runs and reports are in
`analysis/artifacts/d162-2026-11-23/{a,b}-{bench,games}/`; the rule was
applied at full precision by `apply_rule.py` (`rule.json`).

| Change | Bench delta | Games delta | Gap | Band `sqrt(hb²+hg²)` | Rule |
|---|---|---|---|---|---|
| A, table prefetch (`1ff6522` / `b88700a`) | +0.27% (−0.08 to +0.62) | +0.70% (+0.44 to +0.95) | +0.425 | 0.432 | agree |
| B, node work (`b88700a` / `42e3073`) | +1.23% (+0.95 to +1.51) | +1.29% (+0.99 to +1.58) | +0.057 | 0.405 | agree |

**By the frozen rule both changes agree, so the bench read stays the speed
read.** D.16.3 stays open by the maintainer's decision, and its item is read
by the bench read. Change A agrees with 0.007 points to spare. Its two
intervals overlap, which is the looser reading.

What the rule does not decide, recorded for whoever registers D.16.3's
read:

- **On the memory change, the bench read did not resolve the change; the
  game read did.** At step 1, the bench read of A is *between*, with zero
  in its interval. The games read excludes zero and has a narrower
  interval (±0.25 against ±0.35), at 2.6 times the bench's delta. D.16.3's
  item is also a memory change. Its stop rule reverts a read whose
  interval does not exclude zero. On this evidence, a bench read of a
  small prefetch at step 1 can revert a change the games read would keep.
  PROCESS's step 2 narrows the bench read to about ±0.3. Running the games
  read beside it is a choice for D.16.3's registration, made before its
  reads. It is not taken here.
- **The bench read of A differs from its acceptance read.** On 2026-11-20,
  `analysis/artifacts/d7812/nps/read_s1` read +1.09% (+0.52 to +1.67) on
  other pools of the same two sources. Today's +0.27% (−0.08 to +0.62) is
  0.82 points away, against a combined 95% band of 0.67. B's two bench
  reads agree: +0.95% at acceptance and +1.23% today. PROCESS's figure,
  that two pools of one source read within ±0.4%, does not cover A's two
  reads. That points to build luck larger than PROCESS states on this
  change. One instance is not a measured rate.
- **The host's level moved.** The `b88700a` pool's bench level was 2,554,373
  nodes a second today, against 2,418,811 for its predecessor pool on
  2026-11-20 (+5.6%). The cause is not known: the pools and host state are
  both different. The paired deltas carry no level, so the rule is
  unaffected.

**Calibration.** The prediction for agreement held for both changes (P 0.75
and 0.8; both, P 0.6). A's bench delta, predicted at +1.1% (80% band +0.5 to
+1.7), read +0.27%: a miss in magnitude. It leaned on the acceptance read,
which today's pools did not reproduce, so the miss is in the instrument's
reproducibility, not the mechanism. A's games delta, +0.70%, fell inside its
band (+0.5 to +2.2), and the direction predicted, games above bench, held.
B read +1.23% on the bench and +1.29% on games, both inside their bands
(+0.3 to +1.5, +0.2 to +1.6), above the central +0.9.
