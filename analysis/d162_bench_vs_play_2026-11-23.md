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
