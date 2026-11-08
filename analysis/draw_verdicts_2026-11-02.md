# Hard draw verdicts read against the search and against Syzygy, 2026-11-02

PLAN D.6 item 15 (the correction added to the evaluator's draw verdicts) and
D.5 item 19 (the KPK evaluation and `krpkr_scale`'s hard zeros against
Syzygy), run while D.2 waits on the maintainer's night. No engine source
changed: the probe is a diag-only patch built in its own target directory
and reverted; the KRPKR read used a scratch test, also reverted. Raw logs are
in `analysis/artifacts/d5-item15/` (ignored).

## 1. Does the correction move a verdict off 0? (D.6 item 15)

**Instrument.** `analysis/d5_item15_verdict_probe.patch` (applies to
`4441777`) adds, under `--features diag` only, a classifier of the
evaluator's hard verdicts (`draw_verdict_kind`: the drawn KPK, the
wrong-corner KBP, a pawnless minor-material draw, KNNK, and any other
recogniser's zero scale) and counts, at both static-evaluation sites (the
main search and quiescence), every node whose raw value is a verdict and
those whose corrected value is not 0. The corrected value of a raw 0 is the
correction alone (the material scaling and the rule-50 damping multiply the
raw value), so "moved" means the correction moved it. Build:
`cargo build --release --features diag --target-dir target/d5-probe`
(`rarog.exe` `10c7f834…`).

**`bench 13`** (`tools/diag/bench_counters.py --depth 13 --filter verdict`,
`bench13_verdict.log`): 13,987,223 nodes, the fingerprint, so the probe
changes no search.

| Verdict | Leaves | Corrected value not 0 |
|---|---:|---:|
| drawn KPK | 416 | 407 |
| wrong-corner KBP, minor material, KNNK | 0 | 0 |
| other zero scales (KQKP, KRPKR, KRPKB) | 296 | 296 |

703 of 712 moved; 434 by 50 or more; mean |corrected| 108.

**Basilisk's fixtures** (`prime_probe.py` `1c57bb4b…`, `prime_probe.log`),
each arm one process, Hash 8, Threads 1: the four wrong-bishop KBP-K draws
at depths 8 to 14 (`go depth d nodes 200000`), 28 searches, a score beyond
±75 counted as Basilisk's failure.

| Arm | Failures | Verdict leaves | Moved | ≥ 50 | Mean \|corrected\| |
|---|---:|---:|---:|---:|---:|
| primed (one `ucinewgame`, `go depth 12` on the two lost same-structure positions, then the 28) | 24 of 28 (scores +27 to +405) | 542,788 | 99.3% | 379,585 | 117 |
| cold (`ucinewgame` before each search) | 28 of 28 (+123 to +252) | 1,254,210 | 98.0% | 687,051 | 77 |
| organic, `5k2/8/6KP/8/8/8/8/5B2 b` at depths 8–14, cold | 1 of 7 (−110 at depth 14; 0 at 8–13) | 86,161 | 97.6% | 30,217 | 51 |

`ucinewgame` clears the table, the histories and every correction table
(`src/search/mod.rs` `new_game`), and the cold arm's first search ran in a
fresh process: `7k/8/6KP/8/8/8/8/5B2 w` at depth 8 read +123 with 15,545 of
its 16,075 verdict leaves moved. **Unlike Basilisk's, Rarog's verdicts leave
0 without priming:** one search's own training is enough. The priming
searches themselves (mate scores) reach no verdict leaf.

**Reading.** The corrected value leaves 0 at nearly every verdict leaf the
search reaches, by about a pawn on average, in bench and in the fixtures.
That the fixtures' misreads come from those leaves is the repair's
prediction, not shown here: the probe observes, it does not ablate.

## 2. The KPK evaluation against Syzygy (D.5 item 19)

`tests/kpk_syzygy.rs` enumerates every placement of two kings and one pawn,
both pawn colours and both sides to move, keeps what `Board::from_fen`
accepts (it rejects adjacent kings, a side not to move in check and
back-rank pawns), and reads each position's raw static evaluation against
`syzygy::probe_wdl` on the committed `KPvK.rtbw`.

- **662,704** legal positions (Basilisk's count): 445,128 decisive, 217,576
  drawn; no cursed win or blessed loss.
- **Every drawn position reads exactly 0.** A scratch split against the
  bitbase itself (`kpk_split.log`) found the KPK bitbase in agreement with
  Syzygy on all 662,704.
- **88 won positions read at or below 0 for the winner** (0.020% of the
  decisive): 78 of the wrong sign, 10 exactly 0, all where the bitbase
  correctly calls a win and the generic evaluation scores it. 86 have the
  pawn side to move; the pawn stands on its 2nd rank in 68, its 3rd in 16 and
  its 4th in 4, the lone king beside it (`8/8/8/8/8/8/6kP/K7 w` reads −7,
  `8/8/8/8/8/8/2kP4/7K w` −57). Basilisk found 2,022.

## 3. `krpkr_scale`'s five hard zeros against Syzygy (D.5 item 19)

A scratch unit test (recipe below) drew 2,000,000 random legal KRPKR
positions, either side strong and either to move, from xorshift64* seed
`0x9E3779B97F4A7C15`, kept only quiet ones (side to move not in check, no
legal capture or promotion), tagged which `Some(0)` site fired and probed
`KRPvKR.rtbw` (`D:/chess/tablebases/syzygy3456`) without the rule-50
counter (`krpkr_sample_quiet.log`). The unfiltered draw (`krpkr_sample.log`)
is dominated by hanging rooks, which quiescence resolves and a static rule
cannot see.

| Site (`eval/endgame/mod.rs`) | Zeros | Syzygy draw | Pawn side wins | Defender wins |
|---|---:|---:|---:|---:|
| 1 third-rank defence | 48,160 | 46,895 | 926 | 339 |
| 2 checking from behind (pawn on the 6th) | 5,364 | 5,286 | 9 | 69 |
| 3 king on the queening square, rook on the back rank | 873 | 860 | 0 | 13 |
| 4 a7 pawn, a8 rook, king boxed on g7/h7 | 9 | 9 | 0 | 0 |
| 5 king blockades, attacking king far | 11,459 | 11,216 | 202 | 41 |
| all | 65,865 | 64,266 (97.6%) | 1,137 (1.7%) | 462 |

845,892 of the 2,000,000 are wins for the pawn side; 1,137 of them (0.13%)
meet a hard zero. A zero on a defender's win errs toward the truth (the
generic evaluation favours the pawn side), so the defect is the 1,137: sites
1 and 5 carry 1,128, at 1.9% and 1.8% of their zeros. Site 1's relaxation
(`r <= 2 && rank(wr) != 5`, the defending rook anywhere) is the reference's
own approximation; its first failure, `8/1rk5/8/8/8/8/3P1K1R/8 w`, has the
pawn on d2, the attacking king two files from it and the defending rook
not on the sixth rank.

**Scratch recipe** (not committed): a `static KRPKR_SITE: AtomicUsize`
stored with the site number before each of the five `return Some(0)` in
`krpkr_scale`, and an `#[ignore]` unit test in the same module that draws the
positions as above, calls `krpkr_scale`, reads the tag and calls
`syzygy::probe_wdl(&board, false)`; run with
`RAROG_SYZYGY_PATH=D:/chess/tablebases/syzygy3456 cargo test --release --lib
scratch_krpkr -- --ignored --nocapture`. The patch is
`analysis/d4_item19_krpkr_sample.patch`.

## 4. Not read

D.5 item 19's other hard zeros (the KQKP fortress, `krpkb_scale`, the
wrong-corner KBPK) and the partial scales stay unread; KBPvK and KQvKP are
4-man tables, so an exhaustive read of those two costs minutes.
*(Read 2026-11-08: §7. The partial scales stay unread.)*

## 5. The repair read, 2026-11-03 (D.6.15)

Engine commit `3a16fbb`: `corrected_eval_parts` returns `(0, 0)` when the raw
value is 0 and `eval::endgame::is_draw_verdict` holds. Binaries (in
`analysis/artifacts/d5-item15/bin/`): the baseline from the commit before
(`rarog-base.exe`, `b4497868…`, bench 13,987,223 / EBF 2.565), the repair
(`rarog-repair.exe`, `3df6e2bb…`, bench **14,749,043 / EBF 2.564**), and the
probe patch on each (`rarog-probe-base.exe` `10c7f834…`,
`rarog-probe-repair.exe` `d693d315…`).

**Per-FEN differential** (`fen_differential.py`, `fen_differential.log`; a
fresh process per FEN, `go depth 13`): FENs 3 (116,725 → 231,181), 6
(21,127 → 20,461), 37 (123,752 → 76,242) and 40 (148,345 → 180,905) moved,
each reaching verdict leaves in the baseline (278, 19, 58 and 41); FENs 15,
16, 35 and 39 reach a few verdict leaves and did not move; the 32 FENs with
none reproduce their node counts exactly. Confined.

**The probe on the repair** (`prime_probe_repair.log`,
`bench13_verdict_repair.log`): no verdict leaf keeps a moved value, in
bench (644 verdict leaves) or in any fixture arm.

| Arm | Beyond ±75 before | After | Scores after, worst |
|---|---:|---:|---:|
| cleared tables, 28 | 28 | 7 | +103 |
| trained tables, 28 | 24 | 9 | +216 |
| `5k2/8/6KP/8/8/8/8/5B2 b`, depths 8–14, one process | 1 (−110 at 14) | 1 (−110 at 14) | −110 |

The committed test (`tests/endgames.rs`, ignored for D.6.23) reads the
same 7 and 9, and the one-drawing-move position within ±75 at every depth
in a fresh searcher; the probe's −110 came after depths 8 to 13 in the same
process with `ucinewgame` between, so some state survives `ucinewgame`. The
verdict's correction carried most of the misread; what remains comes
through leaves the recogniser does not cover (PLAN D.6.23).

## 6. Measured exposure, 2026-11-03 (D.6.15)

AGENTS (*Gating*, maintainer decision 2026-11-02) asks a repair for the share
of recent gate games that reach its domain. The gates: the three latest
SPRT gates, all on the current evaluation at `3+0.03`
(`tools/results/rar-s105-gate`, `rar-r22-gate`, `rar-r27-gate`; their PGNs
hold 38,860 games, the 38,766 scored and the unfinished pairs). Every game
was replayed with python-chess (`exposure.py` `4510f146…`, `exposure.log`
`aeb6808c…`), every played position with five men or fewer or a bare king
kept as a candidate (416,163 distinct, `candidates.txt` `d33023ec…`), and
each candidate classified by the engine's own `is_draw_verdict` through a
scratch test (`analysis/d5_item15_exposure_classify.patch`, not committed to
the source; `verdicts.txt` `7a78e3bf…`, 13,783 verdicts).

| Share of games | Games | % |
|---|---:|---:|
| the played line reaches five men or a bare king | 17,807 | 45.8 |
| a played position is a verdict | 2,888 | 7.4 |
| a verdict before the game's last position | 1,354 | 3.5 |

The verdicts before the end: a drawn KPK in 556 games, an ending with pawns
(KBP, KQKP, KRPKR, KRPKB) in 317, a pawnless draw in 481. The reach share is
not a bound, since a search from six or seven men reaches five-man leaves
too; the 3.5% is a lower bound on the games whose searches the repair
changes, and the 45.8% the share that enters the domain on the board. Case
1 stands by its rule; no strength gate is registered.

## 7. The remaining hard zeros, 2026-11-08 (D.5.19)

Read exhaustively rather than sampled: every placement of the pieces with
each recogniser's material and pawn precondition, both colours strong and
either to move, kept when legal (kings apart, the side not to move not in
check). Each zero a recogniser returns is probed in `KQvKP`, `KBPvK` or
`KBPPvK` (`D:/chess/tablebases/syzygy3456`) without the rule-50 counter;
quiet means the side to move is not in check and has no capture or
promotion. `krpkb_scale` returns only partial scales (24, 48, 8 on the
`/64` basis), never a hard zero, so it has nothing to read here.

| Recogniser | Positions | Zeros | Syzygy draw | Strong side wins | Defender wins | Quiet zeros | Quiet strong wins |
|---|---:|---:|---:|---:|---:|---:|---:|
| `kqkp_fortress_scale` (pawn on its 7th, any file) | 5,552,520 | 172,256 | 131,256 | 32,360 | 8,640 | 69,760 | 18,628 |
| `kbp_wrong_corner_draw`, one rook pawn | 9,459,860 | 290,880 | 290,880 | 0 | 0 | 233,300 | 0 |
| `kbp_wrong_corner_draw`, two on one rook file | 22,311,920 | 646,296 | 646,296 | 0 | 0 | 446,604 | 0 |

No zero lands on a cursed win or blessed loss. The wrong-corner draw is
exact on every position it fires on. The KQKP fortress is not: 26.7% of its
quiet zeros are wins for the queen side. They sit where the rule's test of
the queen's king, more than two squares from the queening square, is
loosest:

| Queen side's king to the queening square | Quiet zeros, queen side to move | Queen wins | Quiet zeros, pawn side to move | Queen wins |
|---:|---:|---:|---:|---:|
| 3 | 8,052 | 6,656 | 3,544 | 2,176 |
| 4 | 9,628 | 5,896 | 4,404 | 1,684 |
| 5 | 11,224 | 1,316 | 5,260 | 8 |
| 6 | 8,960 | 420 | 4,408 | 0 |
| 7 | 9,500 | 472 | 4,780 | 0 |

At 6 and 7 the wins are all rook pawns with the queen side to move; at 5,
1,104 of the 1,316 are. D.6.24 owns the repair.

**Scratch recipe** (not committed): an `#[ignore]` unit test in the
recogniser tests, `analysis/d519_hard_zeros_scratch.patch` against
`ca8ebcf`; run with `RAROG_SYZYGY_PATH=D:/chess/tablebases/syzygy3456 cargo
test --release --lib scratch_hard_zeros -- --ignored --nocapture` (33 s;
logs in `analysis/artifacts/d519-hard-zeros-2026-11-08/`).
