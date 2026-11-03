# Triumviratus review, 2026-11-03: the third donor, Rarog's checks and the placements

Triumviratus is registered as Rarog's third donor for **mechanisms only**
(maintainer decision 2026-11-03; PROCESS, *The independence boundary*). Its
code and its tuned values do not cross. Its verdicts are **priors, never
acceptance**: each idea is proven or refused by Rarog's own gates.

The reading was done once, in a Basilisk session on 2026-11-03: Triumviratus's
documents, the documents it later deleted, its source, its commit history and
its networks repository. It was not redone here. This packet holds:

- that review's findings, condensed;
- the results of the fifteen checks it asked for, run against Rarog's code at
  `30593ba` (branch `d2`);
- the placement of every finding in Rarog's roadmap, or its refusal with a
  reason.

Ledger import rows: RAR-X10 to RAR-X14 (`docs/EXPERIMENTS.md` §8).

## 1. Sources, pins and the snapshot

| Item | Value |
|---|---|
| Engine | `github.com/Tors3/Triumviratus` `main` at `0c18907e067518d3bb4dc42969eecec671d222d0` (2026-11-03); GPL-3.0; Francesco Torsello, with Maurizio Platino as tester. GitHub's `main` still pointed there on 2026-11-03 (`git ls-remote`) |
| Deleted documents | From `e876fa3370dd8a5c88fcd3b9a911fb94f6911631` (the parent of `5b36328`): `DEVELOPMENT_8.1.md`, `docs/development/8.1/{SEARCH_1_ORDERING,SEARCH_2_REDUCTIONS,SEARCH_3_PRUNING,SEARCH_4_EXTENSIONS,SEARCH_5_CORE,TIME_MANAGEMENT,CONSILIUM_INFERENCE}.md`, `docs/conceptual_maps/BLOCK_{1,5}_CODE_MAP.md`. From `dfa5a6b041b49a9dc554ddb89cb8992846b811fc` (the parent of `9efeed2`): `docs/project/FUTURE_DIRECTIONS.md`, `docs/networks/moe_experimental_grafts.md`, `docs/networks/NETWORKS_4.2-7.0.md` |
| Networks | `github.com/Tors3/Triumviratus-Networks` `main` at `5f3dc60c09dcd7a82449f13f8fbf9a400de04367` (2026-10-28): recipes, run logs, trainer patches. The `.nnue` files are not taken |
| Snapshot (ignored storage) | `analysis/artifacts/triumviratus-reference/`: `triumviratus/` (139 files: README, `docs/`, `source/` without `fathom/` and `incbin/`, `build/`, `.github/`, `removed/e876fa3/`, `removed/dfa5a6b/`) and `triumviratus-networks/` (33 files: every Markdown file and the trainer patches). Copied from Basilisk's snapshot of the same pins |
| Manifests (tracked) | `analysis/triumviratus_snapshot/triumviratus.sha256` and `triumviratus-networks.sha256`, stored byte-exact (`.gitattributes`). Verify with `cd analysis/artifacts/triumviratus-reference/triumviratus && sha256sum -c ../../../triumviratus_snapshot/triumviratus.sha256`, and the same for the networks tree |
| History bundle (ignored storage) | `analysis/artifacts/triumviratus-reference/triumviratus-0c18907-all-refs.bundle`, SHA-256 `79523ab592280776754dc4ea939d387a986fa3a2a4443c3c9a55dd5920cd1ca4`; every branch and tag at the pin (`refs/remotes/origin/main` is `0c18907`). It is the only place the 8.1 commit-message measurements live |

**Why the snapshot is not tracked.** AGENTS' *Evidence* keeps Git for
Rarog's source, tools, fixtures and concise records; a donor's source and
documents are none of these, and Rarog tracks neither Reckless nor
Stockfish. E.3.0's licence decision is also open. So the 172 files live in
ignored storage, and Git keeps what identifies them: the pins, the file
lists and hashes in the manifests, and this packet. This is where Rarog
differs from Basilisk, which tracks its snapshot under `docs/reference/`.

**Verified 2026-11-03, mechanically.**

- Both manifests pass `sha256sum -c` (139 of 139, 33 of 33). The trees hold
  exactly the files their manifests list.
- Every file's git blob id equals the blob at its pin. The main tree was
  checked at `0c18907`, `removed/e876fa3/` at `e876fa3` and `removed/dfa5a6b/`
  at `dfa5a6b`, against the bundle unpacked into a bare repository. The
  networks tree was checked at `5f3dc60`, against a blobless clone from
  GitHub. Result: 172 of 172 files equal.
- The bundle's SHA-256 is the one above.
- Neither tree holds a `.gitignore` or `.gitattributes`, the trap Basilisk hit
  (they act on the host's tree). Basilisk's second trap, a host ignore rule
  dropping a snapshot folder, cannot arise here: the whole snapshot is ignored
  on purpose.

The blob check computes git's id of each file (SHA-1 of `blob <length>\0`
followed by the bytes) and compares it with `git ls-tree -r <pin>`. The
script (`verify_blobs.py`) and the logs (`check_t.log`, `check_n.log`,
`blobs_t.log`, `blobs_n.log`) are beside the snapshot.

**Rebuild without this host.** Clone both repositories, check out the pins,
and copy the listed paths; `git show e876fa3:<path>` and
`git show dfa5a6b:<path>` give the deleted documents. The manifests must
then pass.

**Short references** (paths inside the snapshot):

| Short | Path |
|---|---|
| D6, D7, D8 §n | `docs/development/archive/DEVELOPMENT_6.0.md`, `_7.0.md`, `_8.0.md` |
| D81 §n | `removed/e876fa3/DEVELOPMENT_8.1.md` |
| S1–S5 | `removed/e876fa3/docs/development/8.1/SEARCH_*.md` |
| NOV, NET | `docs/project/NOVELTIES.md`, `docs/networks/NETWORKS.md` |
| FD §n | `removed/dfa5a6b/docs/project/FUTURE_DIRECTIONS.md` |
| R07 | Triumviratus-Networks `recipes/07_consilium-plebis/README.md` |
| commit `x` | a Triumviratus commit message (in the bundle) |

## 2. Decisions carried over

- **Mechanisms only.** Triumviratus's code and tuned values do not cross.
  Its own-lineage networks were trained on public Leela and Stockfish data
  re-labelled with Leela BT4 (NET, *What "own-lineage" means*). Phase F's
  own-data rule excludes that data recipe; its architecture and training
  method may cross as mechanisms.
- **The placement rule, applied here.** On 2026-11-03 the maintainer said,
  for Basilisk: "I do not wish to chase 5 Elo if we are 400 Elo behind."
  Rarog's maintainer instructions say nothing against it, and Rarog's
  position fits it:
  - the C head rates about 3,475 on CCRL's Blitz scale, against the top
    engines' 3,650 and more;
  - the Dragon network in Rarog's own search led the best classical head by
    about +330 Elo (2026-10-15);
  - the network is Phase F.

  So the principle applies. A mechanism measured at about +5 Elo or less
  elsewhere waits for a new final phase, **Phase H — Refinements after the
  big gains**: after the network switch (F.9) and the larger gains (F.7,
  F.8, G.3). Two exceptions:
  - an item that belongs to a defect a leaf already repairs;
  - an item that is a form the leaf must choose anyway.

  Instruments, correctness checks and measurement-integrity reads are never
  small gains. They go to the leaf that consumes them.
- **Instruments first, close to the current work** (maintainer, 2026-11-03).
  The instruments that help now are leaves of the current phase, placed just
  before the next cluster that uses them. Here that is **D.9**, in the register
  after D.4 and before D.5, whose cards are their first users. Its two
  measurement-integrity reads, the wall-clock overhead (D.9.1) and the SPSA
  schedule (D.9.2), are done and clean, so nothing measured so far was
  distorted by them. Basilisk's B.16 is the same step.
- **Own self-play only.** Rarog's networks train on its own self-play, never
  public data. §12 and §12b are read for that case.
- **No board review step.** Basilisk added a board review and rebuild (its
  B.15) against Stockfish's board and Triumviratus's ideas. Rarog's evidence
  says its board does not need one:
  - The board is under a standing contract, *Cross-engine board benchmark
    parity* (`analysis/board_comparison_411b19_2026-09-09.md`). Rarog's
    board measured ahead of Reckless on every workload and close to
    Basilisk.
  - C.12's speed pass already read several of the same ideas by pooled-PGO
    builds and kept them out on measurement. The quiet-SEE skip read −0.58%
    and the SEE fast path a loss (`analysis/c12_speed_review_2026-10-18.md`).
  - SEE and make/unmake are 7.1% and 4.9% of search samples, and the engine
    is memory-bound.
  - D.6.4 and D.6.5 already split the board, behaviour-neutrally.

  Triumviratus's board speed items are placed as leads for H.4, the
  identical-tree throughput pass on the network head. Its correctness
  instrument (TB-1) goes to D.4.17 now. This is not proposed as a leaf;
  a measured board cost on the network head would reopen it.

## 3. What Triumviratus is, and how far to trust it

- **Lineage and standing.**
  - It started from BBC (2024).
  - Up to 8.0 it carried Stockfish's SFNNv16 inference. From 2026-10-04 its
    search sat "on the algorithmic base of Stockfish 19".
  - 8.1 rewrote both in its own form.
  - CCRL 40/15, list of 2026-10-30: 8.0 is first at 4 CPUs (3650 ± 28, 214
    games) and among single-CPU engines (3633 ± 21, 392 games).
- **Development.** It was built with heavy AI assistance; Claude agents are
  named in its logs. Its documents are long and narrative, and they record
  negatives as well as positives. That makes them a better "why" record than
  Stockfish's scattered fishtest links, but a less reliable one.
- **Reading cautions.**
  - One test machine: two Xeon Gold 6138 sockets, one of them with half the
    memory bandwidth.
  - Self-play on UHO 2024, often with hyperthreading. The project found that
    hyperthreading cost 1.6 plies at 5 s and biased SPSA vectors (D8 §45.5).
  - NUMA misplacement hid a network's gain (D8 §9). The two sockets disagreed
    by 20 Elo on the same pair (D8 §29).
  - Many results are early-stopped SPRTs of 1,000 to 8,000 games. Seven of
    thirteen tests peaked between +5 and +17 and ended between −2 and +2
    (D8 §48.1). Its five "leans" together read −0.1 ± 6.7 (D8 §18).
  - Its SPSA tool explored with half the intended perturbation until
    2026-10-13, so every 8.0 tuned vector is rough (D8 §48.1).
  - Its novelty claims are "to our knowledge".
  - Its search is tuned around an NNUE with material experts, at 8 to 40 s
    per game.
- **Its strongest single lesson matches Rarog's method (PLAN rule 3).**
  Every single foreign rule ported into its old search lost or was flat.
  Rebuilding on Stockfish 19's base with a joint 51-parameter SPSA gave
  +85.8 ± 12.8 at 20+0.2 (D8 §16–17). Clusters, not features.

## 4. The fifteen checks, run on Rarog (2026-11-03)

Read from source at `30593ba` with file:line evidence by three read-only
audits. The new claims were spot-checked by hand: the overflow arithmetic,
the SPSA journal, the null-move window and the halfmove parse. Items 1, 3
and 4 were also measured: a release build of `30593ba` (bench 14,749,043 /
EBF 2.564, the declared fingerprint) on an idle host (4% CPU, no engine or
harness running), timed from outside by
`analysis/artifacts/triumviratus-checks/latency.py` and `ponder_trace.py`.

| # | Check | Rarog | Evidence | Disposition |
|---|---|---|---|---|
| 1 | Zobrist key quality | **Correct: GF(2) rank 64** | 793 keys from splitmix64 at compile time (`src/board/zobrist.rs`), all distinct, rank 64. Each part has full rank: the 768 piece keys 64, the 25 side, castling and en-passant keys 25. Triumviratus's 32-bit generator gave rank 32 and false hash hits; its fix read +4.57 ± 3.56 (`24d791c`) | Permanent test `the_hash_keys_span_all_64_bits` (`b028d52`, in `zobrist.rs`). It fails with "left: 32" when the keys are masked to 32 bits |
| 2 | Upcoming-repetition (cuckoo) filter | **Absent** | No upcoming-repetition code in `src/`. Adding Stockfish 19's filter (a cycle counts only when the opponent's key changes inside the window cancel) read +5.62 ± 4.47 over 6,500 (D8 §41). Basilisk's own detection read 0.0 ± 9.3 switched off | D.5.18 already owns the mechanism. The filter is the form it must take there |
| 3 | Wall-clock overhead | **Correct** | One locked `writeln!` per line, no explicit flush per fragment (`src/infra.rs:128-133`). `Instant` with `timeBeginPeriod(1)` (`src/main.rs:103-108`). A persistent helper pool (`src/search/threads.rs:56-73`). Measured, medians of 100 (Rarog / Stockfish 19 / Reckless): `go nodes 1000` 0.55 / 0.93 / 1.52 ms; `go nodes 1` 0.09 / 0.23 / 0.50 ms; `go movetime 100` 100.8 / 100.0 / 86.1 ms (20 each, the timer reading the `info` lines in its own thread of control); with a reader thread draining the output, `go movetime` 50, 100 and 300 overshoot by 0.56–0.68 ms (median) at one thread and 0.78–0.94 at four, 1.44 at most, Stockfish 19's by about 0 (`movetime.py`); at 4 threads `go nodes 1000` 0.28 / 0.57 / 1.05 ms. A ponder that has finished waits in a 1 ms sleep loop (`src/engine.rs:139-149`); `ponderhit` → `bestmove` took 0.53–1.56 ms over 8 cycles, against Stockfish's 0.09–0.11. Triumviratus measured 6.15 ms against Stockfish's 1.24 at `go nodes 1000`, 109.8 ms at `movetime 100`, and 12.3 ms at `ponderhit`; its fix read +29.8 ± 7.4 at 5+0.05 (D8 §45) | **D.9.1, done.** In games RAR-R30 already read it at `3+0.03`: 1.2% of the charged time held after the last `info` line at one thread, median 1 ms. Every gap to Stockfish is under the 1 ms at which it would go to a repair. The ponder wait costs about 1 ms per ponder hit, and no rated list ponders |
| 4 | Re-sent options | **Correct** | A same-size `Hash` re-send costs 0.033 ms, against `isready`'s 0.024 ms floor. Stockfish 19 takes 1.15 ms, and a real resize takes Rarog 1.37 ms. The table is resized only on a change (`src/search/mod.rs:373-375`); the same Syzygy path returns early (`src/syzygy.rs:157-159`); an unchanged `Threads` is a no-op. In Triumviratus a re-send cost one side 0.22 ply and faked a 20-Elo socket gap (D8 §29) | No test pins it. Added to D.4.17 |
| 5 | Null move and the fifty-move counter | **Stockfish's form, a local negative on record** | The null move increments the clock (`src/board/board.rs:1914`), as Stockfish's does, and the evaluation fades with the clock (`eval_rule50_damping`). Triumviratus's fix read +1.6 ± 2.6 on endgames, bundled with a fade formula (D8 §25). Here RAR-S18's null-clock/fence variant read −11.91 ± 7.67. The repetition scan crosses null moves (`board.rs:2291-2306`); D.4.18 records the false twofold this produces | Refused: a +1.6 bundled prior does not fire RAR-S18's trigger. One unverified variant is added to D.4.18: a single null move with the opponent triangulating |
| 6 | Quiescence TT depth | **Correct** | Quiescence stores depth 0 at all three sites (`src/search/node.rs:2257`, `2447`, `2487`). A depth-0 entry never cuts a main-search node and only refines its estimate (`src/tt.rs:699`, `714`). Triumviratus stored −1, and its fix read +3.38 ± 2.58 over 20,162 (D8 §25) | Nothing |
| 7 | SEE: pins and promotions | **Correct** | A pinned recapturer is refused while its pinner stands (`board.rs:1403-1424`; `tests/see_pins.rs`). The promotion gain is counted, and a recapturing pawn becomes a queen (`board.rs:1430-1436`, `1474-1477`; `see_contract.rs` `repaired_recapture_promotion_repair`). Triumviratus read +4.5 ± 6.8 and +1.78 ± 1.76 (D8 §25, D81 §2) | One stale record: `analysis/see_contract_2026-09-06.md` row 32 says no production SEE call reaches a quiet promotion. Quiet promotions are noisy now (`movepick.rs:168`) and reach `see_ge_quiet_aware` (`node.rs:1687`), where the shortcut passes them, harmlessly. Added to D.4.18 |
| 8 | Insufficient material after a capture | **Dead positions only** | The search scores KvK, KNvK, KBvK and same-coloured bishops as draws (`board.rs:2248-2284`, at every node). A minor each is not a search draw. The evaluator returns 0 for any minor against a minor (`src/eval/endgame/mod.rs:205-206`), Stockfish 11's scale, and the search can still find the cornered mates. Triumviratus's set, which adds a minor each with no king in a corner, read +1.13 ± 1.56 over 17,162 endgame games (D8 §42; an idea from Reckless) | The minor-each draw → H.2 (+1.1). Nothing wrong today |
| 9 | Tuning parameters folded in release | **Correct, six reads left** | Search core, proof and quiet parameters, and the evaluation's parameters, are compile-time constants behind zero-sized handles outside `tune` (`src/search/params.rs:37-142`, `src/eval/params.rs:325-357`). `SearchParams` is still a runtime struct: `qs_see_margin`, `qs_see_clamp_hi`, `qs_see_bad_floor`, `nmp_min_non_pawn_pieces`, `probcut_see_gap_scale` and `lazy_margin` load from memory on hot paths (`node.rs:229-232`, `1108`, `2395-2415`; `eval/mod.rs:121-124`). Triumviratus's 303 globals cost it 5.0% NPS (D7 §3) | The six reads go to D.6.11 (one `Compiled<T>`), with exact fingerprint and pooled NPS |
| 10 | Stale per-ply state in a child | **Correct, one rewrite to read** | Rarog has no killers. Every child is bracketed by `push_move` and `clear_move`, and `stack[ply+2].cutoff_count` is reset at entry (`node.rs:725-729`), as Stockfish does. The singular exclusion search and the null-move verification re-enter the same ply and rewrite the stack's `static_eval` and `tt_move` (`node.rs:692-712`); only `tt_pv` and `laterality` are restored. If the correction tables changed in between, the stack's eval can differ from the node's own. Triumviratus's ProbCut child leaked a sibling's state from four of five stacks (D6 note 8); clearing the child ply's killers gave it +12.15 ± 5.97 (D6 row 8) | The re-entry rewrite goes to D.5.5, which owns the exclusion search's estimate. A counter first |
| 11 | TT store race | **Correct** | The shared table is a struct of arrays: six 64-bit payloads and six 16-bit tags per 64-byte cluster. The tag is `key16 ^ fold16(data)`, re-verified against the payload read (`src/tt.rs:181-205`), so a torn pair fails at 16-bit strength. A kept move is reused only from a snapshot verified as the same position (`tt.rs:247`, `862-866`). A probe's move is still checked for legality (`node.rs:651-653`). Triumviratus re-used an entry's move and eval after another thread had rewritten it (D81 §2) | Nothing new. D.2 owns the 16-bit tag's false-hit rate, D.4.17 the torn-pair test (TT-12) |
| 12 | Protocol defects | **Correct, two notes** | Before any `position`, the board is the start position; `d` and `eval` are unknown commands. Lines are trimmed and split on whitespace; input lines have no cap; a clock of −1 or 0 is clamped to 1 ms (D.3.1). There is one stdout lock per line, and only thread 0 prints. `go infinite` waits for `stop`, also when mated. `go nodes` counts every thread. A non-decisive vote never beats a mate; among decisive results the weighted vote decides (a recorded donor divergence, D.2). Thread 0 is reset before helpers start. After a stop mid-iteration, the result line is reprinted when the last line's move differs from `bestmove`, and MultiPV reprints its lines (`search/mod.rs:1436-1456`, `1954-1965`). `currmove` is never printed (optional). Threads get a 16 MiB stack (`infra.rs:112`). The ply cap is D.5.7. **Note 1:** a FEN halfmove clock above 255 is parsed as `u8` and refused, and a refused `position` ends the process (`board.rs:369-372`, `uci_protocol.rs:188-191`). **Note 2:** the correction bonus `corr_update_slope * depth * diff / 128` is `i32` (`src/search/correction.rs:228`). Mate and tablebase residuals are trained on purpose, and `diff` is bounded only by a mate score against an evaluation at the opposite clamp, about 63,700. At that residual the product overflows from depth 135 at the default slope 250, and from depth 66 at the tune range's ceiling, 512. Release has no overflow checks, so a wrapped bonus is clamped to a full-size update of the wrong sign. At the defaults no search reaches it in practice (iterations stop at depth 100); a `tune` arm near the ceiling might. Triumviratus had a 32-bit overflow in its correction decay in tablebase endgames (D8 §12) | Note 1 goes to D.3.9 (PROTO-6's decision). Note 2 becomes a new behaviour-neutral repair, **D.4.23** |
| 13 | FEN robustness | **Correct** | Refused with an error: an en-passant square on the wrong rank (`board.rs:2186-2189`), castling rights the board does not support (`2320-2358`), the side not to move in check (`2163-2166`). An en-passant square enters the key only when a legal capture exists, from a FEN and after a double push (`2205-2208`, `1838-1841`), so one position has one key. Triumviratus's phantom en-passant square gave one position two keys (D6 note 8, `67538a5`) | Nothing. `ep_capture_is_legal` checks slider exposure only; that is exact after a real double push. An unreachable FEN with a knight check and an en-passant square could keep a key with no capture behind it; recorded here, not placed |
| 14 | Time-manager defects | **Correct** | The game ply comes from the FEN's fullmove number (`search/mod.rs:488`, `threads.rs:252`). `go movetime` is used as given (`time.rs:128-143`). The hard stop is held until depth 1 completes (`mod.rs:1786-1791`). Only a `stop` or a tiny node limit before then returns the first legal move unsearched. Triumviratus gave the first move after a book about 30% less time, scaled `movetime`, and could time out before any root move (D8 §20, D6 note 8) | Nothing |
| 15 | SPSA schedule | **Correct schedule; ranges unread** | Colosseum (pinned `8fa63cc`) uses fishtest's form: c₀ = c_end·N^0.102, c_k = c₀/(k+1)^0.102, last iteration exactly c_end (`colosseum-core/src/spsa.rs:120-143`, `272-276`). Rarog's converter sets c₀ to the registered step. The wrapper refuses a horizon that differs from `-Iterations` (`tools/colosseum.ps1:670-671`). Journal `tools/results/rar-r21-fit/run.log`: c 6.0000 at iteration 0, 2.7634 at iteration 1,999 of 2,000, equal to `spsa-schedule.json`'s c_end; `c10-long` 2.0001 → 0.8583 over 4,000. Triumviratus's tool explored with half its intended perturbation for a month (D8 §48.1). Its other rule, start ± 2.25c clear of the points where a formula changes nature (D81 *Plan* item 4), fails at c₀ for 10 of 15 coordinates of RAR-R21's fit and 43 of 82 of RAR-S104's. Examples: `AspGrowthPct` 164 − 2.25·38 = 78.5, below 100, where the window stops growing; five LMR terms cross 0, where they switch off. At c_end, 0 and 7 fail. PROCESS step 8 already notes that a seed within c₀ of a range end biases its estimate | **D.9.2, done.** That journal ran on CLI 0.4.0. The pinned runner is now 0.5.0 (`8fa63cc`), the executable Basilisk pins: Basilisk's live 0.5.0 block on this host (`D:/code/basilisk/tools/results/b33-spsa-block1/run.log`) runs c 1.9999, 1.8634, …, 0.9925 at iteration 961 of 2,000, as c₀/(k+1)^0.102 predicts, so 0.5.0 also ends at c_end. The range rule goes to F.6, the next tune, to settle before its registration; a method change is the maintainer's |

## 5. Search findings (TS; on a Stockfish-19-shaped search)

Rarog's search is Reckless-shaped. "Reckless-derived" marks items whose
mechanism came from Reckless, which therefore matter more here.

| ID | Finding and evidence | Rarog placement |
|---|---|---|
| TS-1 | **Cap on extensions from negative reductions, counted per line** (`NegExtMax=1`). When LMR's reduction is negative, the reduced search runs up to two plies beyond nominal depth, and the excess adds up along a line. One such extension per line, counted in the node frame: −6.6% nodes and −0.55 plies of selective depth at fixed depth (60 positions), +3.88 ± 3.26 over 11,384 at 12+0.12 (D8 §33, §42). Stockfish after 19 bounds the same growth by ply against root depth ("Fix deep recursion") | **D.5.7**, as a form the leaf must choose. Rarog measured the same growth in D.1.6: a third of reduced moves in the cascade iteration run deeper than nominal, outside the line budget, and `CoreLmrDeeperCap` at 0 halves bench (RAR-R19). D.5.7 raises the ply cap; a per-line count or a ply-against-root-depth bound is the other way to keep lines inside it. The +3.88 prior does not justify its own gate |
| TS-2 | **Low-depth hash-move extension (LDSE; Reckless-derived, also Stormphrax)** and its per-line cap (`LdseMax=1`). It read +4.5 ± 3.3 over 11,604 at 10+0.1; the cap read +6.25 ± 7.89 at 15+0.15 (rook endgames used 5–16 times the nodes without it). Switched off for the release after −9.4 ± 10.9 at 40+0.4 with time rules, where the side with it searched 0.1–0.25 ply shallower; switching it off alone at 40+0.4 read −1.15 ± 3.45 (D8 §21–23, §29, §48.4) | Rarog has LDSE (`ldse_applies`, `node.rs:312-321`) inside a line budget: extensions from the root sum to at most the iteration's depth (`within_extension_budget`). That bounds what Triumviratus's cap bounds. Its long-control warning goes to **G.3** (the LTC search fit) |
| TS-3 | **Causal reduction.** After a quiet move fails low, the opponent's refutation marks squares: its from-square, its to-square, and what it attacks from there. Later quiet siblings touching none of them are reduced half a ply more; checks, the hash move and passer pushes are excluded. +4.27 ± 4.13 over 6,594 at 25+0.25 (D8 §38). Variants that lost: `CausalAgree`, `CausalNull` (D8 §46) | **H.2** |
| TS-4 | **TT cutoff value damped toward beta** (from Coda): +2.2 ± 3.2 over 11,774 in three runs (D8 §27) | **H.2**. Rarog returns the stored score (`tt.rs:691-708`) |
| TS-5 | **HistGrad.** The quiet-history update is scaled by the gradient of a softmax over the ordering scores: a surprising best move learns more, and a tried move the ordering rated low is barely punished. +0.90 ± 3.41 over 11,166 (NOV; `59003c8`) | **H.2** |
| TS-6 | **Discovered-check ordering bonus:** +0.8 ± 3.7 over 9,178, paused (NOV) | **H.2** |
| TS-7 | **Chess-knowledge ordering terms.** King-shield malus (Reckless-derived; +3.53 ± 3.42 in 6.0, −1.57 ± 4.32 in 8.0); knight and bishop outposts; attack from a safe square (Reckless-derived; Reckless's own SPRTs +3.7 and +5.7 at 40+0.4); passed-pawn push, queen promotion (Reckless-derived) and bishop-pair capture. Flat at hand values (D8 §34). Baked through an SPSA vector, +3.4 ± 3.9 over about 8,100. On the next network, **removing them read +4.21 ± 2.79 over 15,194, H1**, and saved 1.5–2.4% of cycles (`9910b23`). Counting an ordering bonus again in reductions lost (`KnowRed`, −13.6, D8 §46). An offense-square term read −9.70 alone in 6.0 | **H.2**, a prior against adding them: Rarog's quiet ordering has Reckless's threat escape bonus and threatened-square malus (`src/search/movepick.rs:28-30`), and none of these terms. If one is proposed, it is read only as an SPSA coordinate beside the histories it competes with |
| TS-8 | **Continuous improvement in the late-move count** (Reckless-derived, `ImpSlopeLmp`): the move count scales with the two-ply evaluation change instead of the improving flag. Baked 2026-11-03 with no result in the message (`0c18907`) | Nothing: Rarog's form already. Its move-count limit scales with the improvement's size (`lmp_improvement * improvement / 16`, `src/search/node.rs:1565-1568`), as Reckless's does |
| TS-9 | **No null move with a minor piece or less** (`NmpNpmFloor 3`): +1.32 ± 2.29 over 6,844 on an endgame book (`af49b20`) | **H.2**. Rarog's floor is a coordinate already (`NmpMinNonPawnPieces`, 1, range 1–3), so this is a fit, not a mechanism |
| TS-10 | **Passed-pawn pushes to the 6th and 7th with little material** are reduced less and never pruned: +4.4 ± 6.8 on an endgame book (D8 §17). Older: exempting advanced pawn pushes from pruning read −9.04, then +0.30 ± 2.56 over 22,000 (NOV) | **H.2** |
| TS-11 | **Continuation correction six plies back**, with every correction weight rebalanced (about 0.9×) and the post-null-move correction weight as a lever. Adopted in 8.1 (`2c33b92`); alone it read −0.8 ± 5.6 (D8 §26) | **H.2**. RAR-X04 (a six-ply history channel lost −7.70 in Basilisk) is the local caution |
| TS-12 | **Corrections and margins per network expert.** Pawn and minor correction keys salted with the expert (`CorrPhase`): +3.2 ± 3.8 over 8,634, after first reading −5.1 because a prefetch used the unsalted keys. Per-expert RFP, futility and LMR offsets: +1.6 ± 4.3 over 6,562. Reduction slope per expert: +1.1 ± 3.7. History per expert: −1.7. Piece values per expert unbaked (8.1 against 8.0 at 30+0.3 read +10.7 with them and +20.8 without). Experts 5–7 margins unbaked after −6.49 (D8 §25, §45.4, §46, §47; `c8704f4`, `e1a884c`) | **H.2**, only if F.7 adopts input experts (TN-1). Refused for the classical evaluation: its phase is a continuous taper, not an expert key |
| TS-13 | **Quiet checks removed from quiescence:** +9.71 ± 5.99 over 3,294 at 20+0.2, but only after a one-move-per-node quiescence cap (`QSMoveCap 1 → 3`, itself +5.21) was lifted. Under the cap the sign was reversed (D6 notes 9–10). Stockfish removed them in 2024 (PR #5498) | Nothing: Rarog's quiescence searches no quiet checks. The regime lesson is TM-5 |
| TS-13b | **Also adopted there:** an unconditional check extension removed, +8.0 ± 8.1 in 6.0 (D6 row 4); a continuation correction with a gravity update +13.97 ± 6.43 (D6 row 7); TT16 (TP-1) | Rarog has no check extension (D.5.18 records Basilisk's −22). Its correction already updates by gravity. TT16: TP-1 |
| TS-14 | **Quiescence, negative there:** a passed-pawn push to the 7th searched in quiescence read −0.77, +1.1, −1.8 and +0.23 in four runs; the quiet hash move searched read −1.05 ± 3.92; standing on a material estimate (`QsMatStand`) is off (D8 §21, §25, §41) | **D.5.2** (quiet queen promotions in quiescence): the nearest negative prior for widening quiescence to quiet moves |
| TS-15 | **Root and aspiration, negative or flat there:** a decisive aspiration fail-high (`AspDecisive`, PlentyChess) +0.7 ± 4.2; root depth recovery after a fail-high (Stockfish after 19) −1.82 ± 4.54; a cap on root-move reductions −5.80 ± 5.93; separate window growth after a fail-low and a fail-high, kept as levers (S5 §5.4) | **D.5.13** (the aspiration fail-low window) and RAR-R19's retry row, which already names Stockfish's `failHighRecovery`. These are priors against those forms |
| TS-16 | **Stockfish post-19 ports:** linear razoring at all-nodes −2.32 ± 2.52 over 20,032; a null threshold after sibling null fail-highs +0.46 ± 2.66 over 17,308 (D8 §31) | **D.5.18**, negative priors |
| TS-17 | **Edge cases from other engines, each about ±0.5 over about 7,000 games** (D8 §42): clean singular-verification nodes; a return to the root counted as a draw; the move count restored after the singular search (Reckless-derived); no null move under a hash upper bound | **D.5.18**, null priors: a card needs its edge case shown active here first |
| TS-18 | **Draw value in defence** (`DrawDefend`): +0.3 ± 4.0. Contempt needs a gauntlet, not self-play (D8 §18, §48.1) | Refused: Rarog's contempt card (`analysis/c137_contempt_card_2026-10-16.md`) already reads contempt against the pool |
| TS-18b | **Other negatives** (D8 §18–§47 and its appendix, S1, S2): a null-move threat extension (Botvinnik–Markoff) +2.75, then −0.42; a transition extension (a capture leaving only pawns) −18.2 on UHO; a hash-move refresh on a kept entry −0.65 over 17,574; near-miss TT cutoffs (from Coda) −6.7; no null move, or no RFP improving discount, against an easy capture; a failed null move's refutation used as the node's threat for ordering and reduction; less reduction for double threats; reductions learned from 264M logged reduced searches −1.36 ± 1.44 over 60,000, H0; a quiet history in separate tables by threat context −3.32 ± 1.93 over 33,510, H0 (the factored form is level); pawn breaks in closed structures reduced less, neutral; a cycle counted as a draw only beyond a fifty-move threshold −4.1 and −9.5; Stockfish 19's fifty-move bands in the hash key on an evaluation that already fades −3.7; the correction faded with the fifty-move counter −2.7; an extra evaluation fade with the counter +0.32 over 11,000; correction tables keyed on concepts (king shelter, passer-king zones); "Disagree" (prune less where the network's material and positional outputs disagree), LateBranch, defensive replies, endgame null verification; ordering knowledge in reductions; its own reduction formula and re-search rule (parity at short controls, lost at long controls and on Zen 4) | Negative priors. **D.5.14** takes the fifty-move key band (Rarog's evaluation fades with the clock too, `eval_rule50_damping`, so −3.7 is the nearest prior for BOARD-9) and the correction fade. Rarog's quiet history is Reckless's threat-indexed table: the −3.32 is for separate tables, and the factored form read level. The rest: no placement |
| TS-19 | **Long time-control tree shape.** Stockfish's mean LMR reduction is 6.5 plies against Triumviratus's 3.6 at depth 12 and more in endgames. TT bounds fit the window 66% of the time against 53%; null moves succeed 72% against 60%. No single transplant shortened its tree (Stockfish's endgame reduction base lost 33). A 40+0.4 SPSA reduced less, reached 0.6 ply less, and lost (D8 §10, §19). Depth-dependent levers need a long-control fit; short-control tunes never see depth beyond 15 | **G.3** (the LTC search fit) |

## 6. Board, infrastructure and throughput (TB; speed at an identical tree)

| ID | Finding and evidence | Rarog placement |
|---|---|---|
| TB-1 | **`tdperft`:** perft through the search's own generator, make and unmake. At every node it recomputes every key (hash, pawn, non-pawn, minor and major), the occupancy and the piece table from scratch. It re-checks every parent move against the TT and killer pseudo-legality test. 482M nodes, 0 errors; it validated Chess960 and a board rewrite (D8 §12, §14) | **D.9.3** (an instrument). Rarog's `tests/board_differential.rs` already rebuilds every derived field after each make and unmake, but on random walks, through `generate_legal_moves`. The gaps are exhaustive perft through the staged picker, and every generated move re-checked by the TT-move legality test. It guards D.5.1 and D.5.2, which change the picker's partition, and D.6.4 and D.6.5, which move the board |
| TB-2 | Zobrist rank | §4 item 1: rank 64, now a test |
| TB-3 | FEN robustness | §4 item 13: correct |
| TB-4 | **Per-node state computed once:** pins and checkers shared by the picker's stages; one enemy attack map shared by the generator and quiet scoring (−0.35% cycles, `d6a79a8`); the child's in-check status taken from the parent's gives-check; the illegal-position guard only at the root (D8 §4, D81 §1). A legal generator (from Zoomies/Surge, MIT) in the exact order of the staged one: level on AVX-512, +3.6% to +9.2% elsewhere (D81 §1) | **H.4** |
| TB-5 | **Make and unmake:** unmake restores keys, the fifty-move counter and occupancies from a 64-byte copy taken at make (−0.61%); no branches on captures; minor and major keys kept incrementally (−7.5% instructions with two neighbours) (D8 §4, §40) | **H.4** |
| TB-6 | **SEE kernel:** the next attacker found by index (−0.38%); both bounds of the resolved list tested before any attacker lookup; a quiet move's SEE skipped when the enemy attack map already decides it (28% of calls) (D8 §27, §36; `e876fa3`) | **H.4**. Rarog's own quiet-SEE skip read −0.58% (C.12, row J), a local negative for the third form |
| TB-7 | **Slider attacks:** bishop and rook computed together where both are needed (slider computations per node 9.8 → 5.1); empty-board attacks from a small table; an AVX-512 obstruction-difference kernel on the four lines of a square in one register (−0.98%); hyperbola quintessence on AVX2 (D8 §27, §40; NOV §8) | **H.4**. F.5 records `pdep`-compressed slider tables at −0.98% here (H36) |
| TB-8 | **Generation and ordering mechanics:** a generator templated on the side to move; branch-free selection (the best-quiet selection caused 11% of all branch misses); rank by the count of larger keys, 16 at a time on AVX-512 (−0.70%); captures not regenerated by the quiet stage; the victim read through the mailbox (+0.7% to +1.0%); quiet scoring per node with history rows read once (−2.6% instructions); vectorised quiet scoring on AVX-512 (−2.38%, `9910b23`) (D7 §3, D8 §4, §20) | **H.4** |
| TB-9 | **Chess960:** occupancy updated by XOR (king and rook may share squares); both origins cleared before both destinations; the rook origin from one table; X-FEN and Shredder-FEN; castling written as king-takes-rook; the FRC perft suite. No speed cost (D8 §14) | **G.2** (Chess960 on demand) |
| TB-10 | **One board for search and network:** make records only the moved piece; before an evaluation the pending moves are replayed from the current board (+0.6% to +1.1% instructions against a lazy second board); a branch-free refresh-cache comparison (D8 §28) | **F.1** (board events and accumulator ownership) |
| TB-11 | **Stack probes** (`__chkstk`) in hot functions with frames over 4 KB on Windows: list fills moved out of line and evaluation lists off the stack, −1.55% cycles (D8 §22) | **H.4**. Rust emits the same probes; D.4.10 (stack-built tables) is the nearest open item |
| TB-12 | **Prefetches at make:** the child's TT entry from a predicted key (+2.21% NPS); the child's correction entries (+2.04%); the continuation-correction cells (−0.3%, which also removed a memory-bandwidth asymmetry) (D8 §4, §48.2). The next-move TT prefetch cost on AMD and on the dual Xeon (D81 §1) | **H.4**. Rarog's B.7.2 audit read both prefetches locally (`analysis/b72_audit_a_tt_prefetch.patch`, `b72_audit_i_correction_prefetch.patch`); D.2 owns the fetch lines for moves later pruned |
| TB-13 | **Search state on large pages** (histories, corrections, per-thread data) beyond the TT: −0.93% cycles (D8 §30) | **G.1** (large pages and placement) |
| TB-14 | **Builds:** ext-tsp block layout, and no cmov-to-branch conversion at link (−0.15%, −0.29%); deterministic PGO training at fixed nodes, so two builds of one source are identical (D8 §31, §36, §48.3); a universal binary from one unit per source file with ThinLTO and merged per-variant profiles (−0.28% against separate builds; one unit per variant cost +1%) (D8 §35); native PGO per tier (+1.3% on AVX-VNNI, D81 §1); an AVX-VNNI tier for Intel 12th generation and later (+6% to +9%, NNUE) | The universal binary and per-tier PGO: **G.2**. The AVX-VNNI tier: **F.5**. Layout flags: **H.4** |
| TB-15 | A per-thread evaluation cache removed because the TT already stores the evaluation (12 hits in 2.5M lookups) (D81 §1) | **F.5**: Rarog's evaluation cache pays for the classical evaluation; the network changes its value |
| TB-16 | **What did not pay on that Xeon:** prefetching and reordering loads, mostly; AVX-512 gathers (as costly as sixteen scalar loads); the AVX2 table-driven compaction (D8 §20, §23, §27) | **H.4**, negative priors |

## 7. Time management (TT)

| ID | Finding and evidence | Rarog placement |
|---|---|---|
| TT-1 | **Loss analysis by cell.** A judge engine scores every move of lost and spoiled games, tabulated by position class (defending, level, ahead) × clock left × material. 33.8% of its loss against Stockfish 19 sat in "defending with under 10% of the clock", and half the real errors vanished with more nodes (D8 §32, §43). It held 36.7% of its defending games against Stockfish's 50.6% (D8 §47.4) | **D.9.5** (an instrument): on stored pool and gate games, judged by Stockfish 19 at fixed nodes, both sides, the worst errors re-searched at 4× and 16× nodes to split clock errors from evaluation errors. It says where the classical head's losses sit, and its table ranks H.3; E.1 re-reads it at the release head. Rarog's conversion instrument covers only won positions |
| TT-2 | Wall-clock overhead | §4 item 3: correct, measured |
| TT-3 | **Measure each time lever at the time control it acts at.** Tune against an external engine with release-speed builds, because self-play rewards the side that spends more (D8 §19, §21, §23) | **H.3**, as its rule. D.1.5's harm reads at two controls are the local form |
| TT-4 | Time-manager defects | §4 item 14: correct |
| TT-5 | **Negative:** a lower cap on the longest thinks lost −27.1 ± 17.0 at 40+0.4; an expected-reply depth floor −5.5 ± 13.4 (D8 §23, §27) | **H.3**, negative priors |
| TT-6 | **A moves-left horizon fitted on its own games** (`TmOwn`), at parity. SPSA'd time levers were ramped by game time between fitted values (at 21 s or less) and SPSA values (at 35 s or more), because the vector lost at 8+0.08 and gained at 25+0.25 (`5a1a1b2`, `67538a5`). Earlier, TMv2's multiplicative factors (Alexandria, Caissa) read +23.8 at 20+0.2 (D6) | **H.3** |
| TT-7 | **Surprise rule:** 1.2 times the time when the opponent does not play the predicted reply. +6.23 ± 6.75 at 40+0.4, +0.7 at 16+0.16; silent while behind on the clock (D8 §23; `ce03c53`). Caissa publishes both directions (0.915 and 1.132) | **H.3** |
| TT-8 | **Less time when behind on the clock:** optimum × (1 + 0.6·min(relative clock advantage, 0)). +5.23 ± 4.72 at 6+0.06, +0.36 ± 5.13 at 30+0.3 (D8 §31). Stockfish added a similar rule after 19 | **H.3** |
| TT-9 | **Time in defence and early saving:** 1.5 times the time while the root score lies in about [−3.6, −0.45] pawns, and the optimum cut by 20% with 24 pieces or more. +7.27 ± 7.08 at 40+0.4 together, never separated (D8 §33, §43) | **H.3**. TT-1's table on Rarog's games decides whether defence is where its time goes wrong |
| TT-10 | **Smaller rules:** a guard against saving time on an unconfirmed jump of two printed pawns (−2.52, kept as a guard); a graded surprise; more time when the PV trades into another expert (`TmTrans` +2.35 ± 4.21); a time share per expert (+1.9) (D8 §44, §47; FD §6) | **H.3** (the expert rules only if F.7 adopts experts) |

## 8. Threads, TT and SMP (TP)

| ID | Finding and evidence | Rarog placement |
|---|---|---|
| TP-1 | **TT16:** 16-byte entries, four per 64-byte bucket, **XOR-verified and lockless**. A 48-bit key tag and the static evaluation share one word, XORed with the data word, so a torn write fails the check. The bucket index is a multiply-high, so a `Hash` that is not a power of two is used whole. +6.30 ± 5.06 at Hash 16 against its own 24-byte, two-way layout that straddled cache lines (D8 §6; `source/tt.h`) | The index: **D.5.12** (`Hash` at any size), which already names the multiply-shift form. The verification width: **D.2**, which owns the 16-bit tag's false-hit study. The layout gain does not transfer: Rarog's clusters are 64-byte aligned and never straddle |
| TP-2 | TT store race | §4 item 11: correct |
| TP-3 | **Per-thread state `alignas(128)`** against false sharing at every make and unmake (`f7e1489`) | **D.2**, beside its `repr(align(64))` item for the shared counters |
| TP-4 | **SMP levers:** a helper LMR bias in fine units, +4.91 ± 7.97 at 8 threads (Reckless-derived, D6 row 5); a thread vote −3.5 ± 8.7; optimism per thread +0.9 ± 8.6 (D8 §12); staggered helpers and depth in the vote +1.72 ± 4.61 (S5 §5.3); a helper one iteration ahead at ply 2 of the PV, time to depth 0.908 at 8 threads with no trend with depth, −2.23 ± 15.48 (D8 §47.2). Its 1→4 CPU CCRL step (+27 to +33) matched other top engines (D8 §12) | **D.2**, priors for its engine card if RAR-M83 reads DEFICIENT; otherwise **G.1** |
| TP-5 | SMP contracts | §4 item 12: correct |
| TP-6 | **NUMA:** threads spread over nodes; one network copy and one set of shared histories per node; each thread's network handle on its own node (`f7e1489`, `36c4e9f`, `f7a83be`) | **G.1** |

## 9. Measurement and tuning method (TM)

| ID | Finding and evidence | Rarog placement |
|---|---|---|
| TM-1 | One engine per physical core (§3's cautions) | Already Rarog's policy for rated multi-thread reads (RAR-M82, PROCESS). Corroboration only |
| TM-2 | SPSA schedule | §4 item 15: correct |
| TM-3 | **A lever enters a bake only when its net move is at least 2.5 times the random-walk expectation.** Random-walk segments look monotone (rank correlation 0.85 or more) 30% of the time, so with 37 levers about 11 pass by chance (S2 §2.4) | **F.6**, to settle before its tune's registration. Rarog's movement rule (rule 7c) counts coordinates moved at least one step, which is the same question asked against a fixed step |
| TM-4 | Stopped leans add up to nothing (§3) | Rarog's rule already: an unresolved stop is not "probably fine" (AGENTS, *Gating*) |
| TM-5 | **A measurement is valid only in the regime it was taken.** Quiet checks in quiescence flipped sign once a cap was lifted (D6 notes 9–10). A correction weight moved up when tuned at 15 s and down at 30 s, and only the 30 s vector passed (D7 §2) | **F.6** and **G.3**, as a registration check |
| TM-6 | **Analyse an idea that should not lose but does, before closing it.** `CorrPhase` lost until a prefetch of the wrong keys was found (D8 §25) | Rarog's practice already (RAR-S64; the research packets' falsifiers). Corroboration |
| TM-7 | **Speed by hardware counters at an identical tree:** cycles, instructions and branch misses per node, engine pinned, search threads only (start-up was 5% of cycles at 400k nodes per position); a byte-identical copy of the base as the noise control; paired simultaneous runs on two hyperthread siblings, null test ±0.1% (D8 §2, §36; `build/nps_pair.py`, `build/node_identity.py`) | **D.9.6**, beside Rarog's pooled-PGO method, which already uses a self-pair as its control; its card chooses the tool on the 5950X and asks whether code layout forces several builds here too. Its users are D.4's and D.6's NPS guards, D.8 and H.4. No sibling pair: RAR-P54 read both engines at 0.63–0.65 beside a sibling |
| TM-8 | **Tree-size centring for any lever that changes the tree.** A fit on search data plus a node-count sweep on fixed positions finds in minutes what an SPSA looks for over hours. The bench is useless for it, because a few positions dominate its sum (S2 §2.6) | **F.6**: with rule 8's curvature sweep, which reads `bench 13` and WAC. The "top-pos share" line (14.4% today) is the bench's own warning |
| TM-9 | **Pricing what pruning and reductions hide.** Data builds record which rule pruned or reduced a move, and search a sample of those moves again at the denied depth: `prunelog` for every pruning rule (S3 §3.3; `source/prunelog.h`), `redlog` for reduced searches, with a quarter of reduced fail-lows probed at full depth (S2 §2.6; `source/redlog.h`); `ordlog` is a move-ordering oracle (`source/ordlog.h`). The label that matters is "failed low reduced, beats alpha at full depth" (1.7% of probed moves). Its first label, "beats alpha", spent nodes on moves already re-searched (−9.65 ± 10.36) | **D.9.4**. Rarog's `diag` counters count each rule's decisions and its ablation mask prices whole mechanisms on fixed-node screens; nothing prices the moves a rule denied. The card settles the form, because the `diag` build may not change nodes and a re-search writes the TT and histories |
| TM-10 | **Shortcut builds.** Every identical-tree shortcut ships with a build switch that runs the original beside it at every node and aborts on the first disagreement (`TRIUMV_VERIFY_*`: SEE, TT store, rank, quiet scores, attack maps, accumulator sync) (D8 §27, §40) | **H.4**, with each shortcut it reads. Rarog's exact fingerprint catches a shortcut that moves the tree, but not one that disagrees only off the bench's positions |
| TM-A | **Ablations:** switching off features accepted on weak evidence, with a simplification SPRT, freed two features (`QuietOffense`; `ContHistPrune` +5.94) (D8 §8) | **H.1**: the triage ablates the features accepted on weak evidence |

## 10. Evaluation analysis (TE; useful for a classical evaluation)

| ID | Finding and evidence | Rarog placement |
|---|---|---|
| TE-1 | **Concept residual regression** (FD §1). For game positions, take the score of a depth-1 search with quiescence against a 300k-node search. Regress about thirty concepts on that residual by least squares, holding the starting score, material and piece count fixed: passer rank, unstoppable passer, connected passers, space, open files by the king, king-zone attackers, locked pawns, drawing factors. \|t\| > 3 counts as a signal. On its network: passers +11.1 cp per rank in endgames (t 10.3), unstoppable passers +44.8, space +12.0 per square, open files by the king −15.4, king-zone attackers −11.1; drawing factors handled. For an NNUE every fix outside the network then lost (D8 §44). For a classical evaluation, a signed residual points at a term's value | **F.2.1** (the classical refit on search-visited data), as a screen on F.2's positions before the fit. Rarog's `tools/diag/donor_residual.py` is the nearest local instrument |
| TE-2 | **Unsupervised regimes.** First-layer outputs are clustered and compared with candidate keys by normalised mutual information: material × queens 0.43, queens 0.39, material bands 0.34, pawns 0.29, closed structure 0.07 (FD §2). Useful for choosing NNUE bucket keys | **F.7** (bucket keys) |

## 11. NNUE architecture and inference (TN)

| ID | Finding and evidence | Rarog placement |
|---|---|---|
| TN-1 | **Input-layer experts by material (Consilium).** The king-relative block in four weight sets by total piece count (bands ≤9, 10–15, 16–23, ≥24, equal-frequency), trained as a shared base plus zero-started per-band deltas (the factoriser applied to bands). A capture across a band forces a refresh, with a refresh cache per band. −7.5% NPS, against −11.2% for an L1 of 1536 and −20% for 2048. About +15 of 8.0's +27.3, with its retune (NET, NOV §1) | **F.7** |
| TN-2 | **Per-side experts (Consilium Plebis):** eight per perspective, chosen by the opponent's non-pawn material through a routing table that was learnable but moved under 1% of its cells; int8 king-relative weights clamped during training; a pawn-king subnetwork (256 hidden, cached per thread, added to the first layer of each of 8 layer stacks). +10.3 ± 4.3 over 8,332 at fixed nodes (NET, R07) | **F.7** |
| TN-3 | **Input blocks:** `PassedPawns` (96 inputs, one per passed pawn by square), +6.96 ± 6.56 grafted; `PawnPair` (Pawnocchio's, now Stockfish's `PP_3Wide`), +18.27 ± 9.94 with own self-play data; threat inputs (SFNNv13 onward); a mobility block, −16.2% NPS untrained (NET, D6, D8 §5) | **F.7** (relation and threat inputs) |
| TN-4 | **A zero-initialised graft** as a cheap feature screen, with its limit: frozen later layers may not use new information. A passed-pawn state block gained only once the layers after the accumulator were unfrozen, and a block started from existing rows needs a lower rate (NOV §7, D8 §38) | **F.7**, as its screening method |
| TN-5 | **Lost:** averaging two experts near a band edge, +5.27 at fixed nodes but +15% to +18% cycles and −24.23 at time; a split accumulator, +12% cycles from doubled memory traffic; 8-bit king-relative weights quantised after training, 0.2–0.5 ply slower (it needs clipped training); fixing the root's expert for the whole search, −27.7 (D8 §44, §46; `878b574`) | **F.7**, negative priors |
| TN-6 | **A policy network inside alpha-beta lost in four forms** (−85, −30.56, −46.89, −22.83). Ordering already reached 79.2% first-move cutoffs without a hash move, and a static prior computed at the parent loses to histories that adapt to the subtree (NOV §9) | Refused: a negative prior, recorded in F.7 so it is not proposed |
| TN-7 | **Inference speed:** a refresh cache per band; a pawn-block refresh cache keyed by the pawn bitboards (+2.8%); the hybrid update on king moves (Stockfish `db98633b`, +3.4%); both perspectives in one dirty-list pass (+1.5%); first-layer neurons permuted at export so zero activations fall in skipped blocks (+2.5%, bit-identical); no threat tuples opened and closed inside one capture (`b34547f`); the System V calling convention for inner functions on Win64, which avoids saving xmm6–15 (`8568809`); the network on large pages (+6.7% at 250 MB, `42e68f5`); the threat-row layout by read frequency (D7 §3, NET) | **F.5** |
| TN-8 | **Memory is the cost:** 47% of last-level misses fall in the incremental accumulator update, 11% in refreshes, and under 5% in corrections, ordering and the TT together (D8 §46.3) | **F.5** |
| TN-9 | **Conformance gates before training:** an integer evaluator matching the trainer on exported nets; loader indices against python-chess (0 of 7,025 wrong); a random net with perturbed per-band deltas, since zero deltas hide a wrong index; the engine against the evaluator move by move across bucket changes; incremental against refresh at every node (`nnperft`); a golden set of 2,000,124 positions bit-identical after a clean-room rewrite (NET, R07; `c35fcca`) | **F.4** (integer-exact conformance) and **F.5** (incremental parity): instruments |

## 12. NNUE data and training (TD)

| ID | Finding and evidence | Rarog placement |
|---|---|---|
| TD-1 | **Watch the holdout, not the training loss.** A λ cycle that climbed back to 1.0 made the network imitate its labels. On a fixed holdout the λ=1 loss kept falling, while the λ=0 (game result) loss stopped at epoch 200, exactly where Elo flattened. A fixed λ of 0.75 restored the gain. Watch holdout loss at fixed λ=0 and λ=1 (NET) | **F.3** |
| TD-2 | **Register checkpoints with Elo targets during training.** An early logarithmic fit mispredicted the plateau (−32 predicted, −70 measured); late gains arrive as the rate decays (NET) | **F.3** |
| TD-3 | **Restarts and averaging.** A restart cycle at a fifth of the peak rate moved a plateaued pretraining. The final net is the average of the last few epochs' weights; a second cycle after that made it worse (NET, R07) | **F.3** |
| TD-4 | **The loader limits throughput:** one feature thread per GPU starved it (381k → 1.9M positions/s with a larger share); files drawn uniformly over-read small files; piece-count sampling shifted toward endgames fed the endgame experts; soft rather than hard early skipping; about $0.17 per billion positions on rented 4× RTX 5090 (NET, R07) | **F.3** |
| TD-5 | **Converged networks barely move.** More epochs on data of the same family moved a converged net by 1% in scale and 0 Elo; per-expert fine-tunes read +0.7 (D8 §29) | **F.8** |
| TD-6 | **Proposals, unmeasured** (FD §3–5): hard examples (positions lost, or where a deep search contradicts the static score) need a dedicated generation at scale; overlapping material bands in training; shared and private first-layer lanes per expert | **F.8** (hard-position mining); the expert proposals in **F.7** |
| TD-7 | **Gains hide.** Network gains shrink at longer controls and hide behind shared cores: one net read +0.8 ± 10.1 at 30+0.3 with 75 concurrent games, and +23.4 ± 12.0 at 12+0.12 with 38 (D8 §9) | **F.3** and **F.7**, as a measurement rule: Rarog's sibling-free placement (RAR-M82) is the local form |
| TD-8 | **Recalibrate per network:** the evaluation scale per piece-count bucket, judged by games (matching the raw ratio to the previous net lost); scale-dependent margins (25 parameters) retuned on the final net (+8.6); blend constants inherited from Stockfish, tuned on its own net (+3.75 ± 3.49 at 20+0.2) (NET, D7 §2) | **F.6** |
| TD-9 | **One label scale per run, and labels strong enough.** Triumviratus re-labelled every source with one evaluator before mixing, because mixed targets train toward neither. A net trained on Stockfish's 5,000-node self-play alone ended −84 against the reference, and a second stage on data with stronger labels closed half the gap (`removed/dfa5a6b/docs/networks/NETWORKS_4.2-7.0.md`; recipe 02) | **F.2**, read for self-play: data from successive heads is re-scored by one search before mixing, and the node budget per label is sized by the learning curve, not by throughput |
| TD-10 | **Measure the stream's shape:** files drawn uniformly over-read small files; sampling by piece count toward endgames, which material buckets otherwise starve; soft rather than hard early skipping; the positions the search actually evaluates compared with the training stream bucket by bucket. On Triumviratus the search held 2–3.5× more unbalanced material pairs than the data, and game positions at every ply were the right source; book exits and endgame books were not (recipe 07 `docs/C2_SEARCH_DISTRIBUTION_2026-10-21.md`) | **F.2** |
| TD-11 | **Its first network's failures, each visible in the command line:** no factoriser, λ 1.0 only, and a stop on a flat validation loss while the rate was still 38% of its start read −102 against the reference; fixed, −39 (recipe 01) | **F.3**: every recipe checked against a reference recipe before a run is paid for |
| TD-12 | **Judge checkpoints carefully:** at 20+0.2 or longer (one net read −27 at 8+0.08 and −84 at 20+0.2); against one fixed anchor (checkpoint-to-checkpoint deltas summed to +71 where the direct gap closed by +28); the first epochs after a restart at a higher rate are recovery, not progress (−27 at epoch 19, parity by 119); a validation loss computed at a moving λ cannot rank checkpoints, so save-top-k by it is unusable (recipes 02, 05) | **F.3**; F.8 gates each refresh on the same anchor |
| TD-13 | **Saturation calls for capacity or information, not epochs.** On the 1024-wide net there was no gain between epochs 249 and 416, an annealing tail read −0.28 ± 5.21 over 5,024, and fine-tunes read zero. Meanwhile a 99k-parameter passed-pawn block bought +7 in four epochs, and material buckets about +15 (recipe 05) | **F.7** |
| TD-14 | **Own self-play in a refine:** 67M positions (6.5% of the mix, listed ten times) rode with new Leela data and the pawn-pair block, +18 together, so their own share is unknown. A filter that streamed only passed-pawn positions added nothing to a converged block (recipe 03) | **F.8** |
| TD-15 | **Finishing:** the weight average of the last few epochs (+3.8 ± 7.7 over a single checkpoint, kept for its lower variance). Feature-transformer neurons permuted over **all** input blocks, the evaluation bit-identical (+1.6% to +2.5% NPS); permuting only some blocks makes the net wrong, not slow | The average: **F.3**. The permutation: **F.5** |
| TD-16 | **Shape of a network step** (recipe 07 `docs/DESIGN_NEXT_NETWORK_2026-10-21.md`): CPU-only gates before any GPU hour (the shape's speed with a random net, bucket changes per node in the tree, occupancy per bucket on the data, engine against trainer); then proxy trainings of about 40G positions, each judged by an SPRT; then one full run, close calls taking the conservative choice. The same document surveys Stockfish's networks in October 2026: PSQT removed (SFNNv17); int8 first-layer weights trained by clamping the merged weights; king × opponent-queen buckets with 32 layer stacks (SFNNv18, about +4 at STC and LTC, every single change failing alone); row pairing in the update (+5% to +7% NPS); learned routers for the layer stacks; pawn input blocks in test | The step shape: **F.0** and **F.7**. The survey: **F.0**, for F.7 |

## 12b. Triumviratus's network path, net by net (read for a self-play network)

| Net (release) | What was done | Result | Lesson |
|---|---|---|---|
| before 4.2 | Stockfish's own network in the engine; a policy network tried inside alpha-beta | the policy net lost in four forms | TN-6 |
| `rubicon-v1` (4.2) | `HalfKAv2_hm^`, L1 2560, Leela `test80`, nnue-pytorch; the first run without the factoriser, λ 1.0, stopped at epoch 124 on a flat validation loss | first run **−102** against the Stockfish net; the fixed run (factoriser, about 400 epochs, λ squeezed to 0.75 on the best checkpoint) **−39** | TD-11 |
| `rubicon-alea-v1` (5.0) | threat inputs (SFNNv13), L1 1024; stage 1 on Stockfish 5,000-node self-play, stage 2 on Leela data at λ 0.74 | stage 1 **−84** at 20+0.2 (−27 at 8+0.08); after stage 2 **−40**; the engine +50 over 4.2 | TD-9, TD-12 |
| `rubicon-alea-v2` (6.0) | v1 fine-tuned with a pawn-pair block grafted at zero, two learning rates (base 1e-4, new block 1e-3), λ 0.75 fixed, 6.5% own self-play; validation flat from epoch 80, stopped at 470; annealed, the last three averaged | **+18.3** over v1, the network isolated | TN-4, TD-14, TD-15 |
| `rubicon-alea-v3` (6.0) | a 96-input passed-pawn block trained alone on the frozen base | **+7.0** in four epochs; later epochs no better | TD-13 |
| `legio-septima` (7.0) | SFNNv16, all from scratch, both stages re-labelled to one scale; λ fixed early in stage 2 | **+23.4**; then saturated | TD-9, TD-13 |
| Consilium (8.0) | four input-weight sets by material at the same L1; one data mix; a λ cycle climbing back to 1.0 caused a plateau, fixed mid-run; restart cycles; the last five epochs averaged | about **+15** of 8.0's +27.3 | TN-1, TD-1, TD-3 |
| Consilium Plebis (8.1) | eight per-side sets by the opponent's material, int8 king-relative weights, a cached pawn-king subnetwork; CPU gates and a staged plan before training | **+10.3** at fixed nodes | TN-2, TD-16 |

**What a self-play-trained Rarog can skip, in the order Phase F meets it.**
Each is now a requirement of the PLAN leaf named (PLAN Phase F, *Lessons from
Triumviratus's network path*):

1. a recipe checked against a reference before the first paid hour (F.3; TD-11);
2. conformance gates before training (F.4; TN-9);
3. one label scale, and labels deep enough (F.2; TD-9);
4. the stream's shape measured against what the search evaluates (F.2; TD-10);
5. checkpoints judged at long controls against one anchor, on holdout loss at
   fixed λ, with λ fixed early (F.3; TD-1, TD-12);
6. capacity, not epochs, once saturated (F.7; TD-13);
7. each network step as CPU gates, proxies, then one full run (F.0, F.7; TD-16).

## 13. Analysis build and product (TA)

| ID | Finding and evidence | Rarog placement |
|---|---|---|
| TA-1 | **Analysis build ("Position Solver"):** the second root move always searched with a full window (Arasan 157 → 170 at 1M nodes, two to three plies of depth), paused in quiet positions; the more direct of two clear wins played, with the search tree unchanged; the analysis search compiled as a second copy, so games pay nothing (D8 §36, §49) | **G.2** (product) |
| TA-2 | **Displayed scores:** the printed scale calibrated on self-play so that +1.00 is a 50% chance to win, refitted per network; tablebase wins printed as `cp 20000` minus the distance (D8 §24; `28c44db`, `9e74ae8`) | **D.3.9** (the score-normalisation card). Rarog already maps the tablebase band to `cp` 19,873–20,000 |
| TA-3 | **Strength levels** (`UCI_LimitStrength`, `UCI_Elo`): limit the search and choose among the best root lines, calibrated in games (FD §13) | **G.2** |

## 13a. One rebuild or clusters, read for Rarog

Triumviratus did both, and its record separates the cases.

- **One step with statistical acceptance.** It rebuilt its whole search on
  Stockfish 19's base in a day, fitted it with one 51-parameter SPSA, and got
  +85.8 ± 12.8 at 20+0.2 (D8 §17). It then spent two weeks finding, one SPRT
  at a time, what the big diff hid: a cycle filter from an older Stockfish
  (+5.62), the quiescence TT depth (+3.38), pins in SEE (+4.5), the null
  move's fifty-move counter (+1.6), a lost prefetch, seven adapter
  divergences and wrongly ported options (D8 §20, §25, §41, §48.2). Its
  one-step rewrite of the reductions in 8.1, accepted on short-control
  parity, lost at long controls and was taken back out (D81 §3).
- **One step with exact acceptance.** Its 8.1 rewrites at an identical tree
  (inference, infrastructure, the time manager, each search block) went fast,
  because bench, perft and bit-identical evaluation decide each one without
  games.

**For Rarog** this is PLAN rule 3 and AGENTS' behaviour-neutral rule, with
evidence from a third engine. A component with an exact oracle (an identical
tree, perft, bit-identical output) may be rebuilt in one step: D.6's
simplification programme, D.9.3's perft as its guard, F.4's and F.5's
integer-exact conformance. A component accepted by games is cheaper in
clusters, because a failed one-step change cannot say which part failed, and
constants on another engine's score scale must be fitted cluster by cluster.
Rarog's own first cluster read +65 unfitted and +138 fitted (RAR-S73). Nothing
new is placed.

## 13b. The instruments step: PLAN D.9

The instruments that help now, placed just before the next cluster that uses
them (§2):

| Leaf | Instrument | State | First users |
|---|---|---|---|
| D.9.1 | Wall-clock overhead read (TT-2, §4 item 3) | DONE 2026-11-03, clean | every `3+0.03` read so far |
| D.9.2 | SPSA schedule read (TM-2, §4 item 15) | DONE 2026-11-03, clean | every fitted vector so far; F.6 |
| D.9.3 | A search-path perft (TB-1) | READY_FOR_IMPLEMENTATION, `I1` | D.5.1, D.5.2, D.6.4, D.6.5 |
| D.9.4 | Pruning and reduction pricing (TM-9) | RESEARCH, `R2` then `I2` | D.5.3, D.5.5, D.5.16, D.5.19, F.6 |
| D.9.5 | The loss table by cell (TT-1) | READY_FOR_IMPLEMENTATION, `V` | E.1, H.3, the open draw-rate question |
| D.9.6 | An identical-tree counter read (TM-7) | RESEARCH, `R2` then `I1` | the NPS guards of D.4 and D.6, D.8, H.4 |

## 14. Placements by Rarog leaf

| Leaf | Items | Kind |
|---|---|---|
| D.2 | TP-1 (verification width), TP-3, TP-4 | priors for its card; TP-4 moves to G.1 if RAR-M83 reads NOT DEFICIENT |
| D.3.9 | §4 item 12 note 1 (halfmove clock above 255), TA-2 | decisions |
| D.4.17 | §4 items 4, 10 and 12: a same-value `Hash` keeps the table, repeated spaces and a trailing `\r`, a 64 KB `position` line, a clean frame on every child entry path | tests |
| D.4.18 | §4 item 5 (the single-null variant, unverified), §4 item 7 (the stale SEE record) | record corrections |
| **D.4.23** (new) | §4 item 12 note 2: the correction bonus in 64-bit arithmetic | behaviour-neutral repair |
| D.5.2 | TS-14 | negative prior |
| D.5.5 | §4 item 10 (the re-entry rewrite) | a counter first |
| D.5.7 | TS-1 (and TS-2's per-line cap, already bounded here) | a form the leaf must choose |
| D.5.12 | TP-1 (multiply-high index) | the leaf's form |
| D.5.13 | TS-15 | negative priors |
| D.5.14 | TS-18b (fifty-move key band, correction fade) | negative priors |
| D.5.18 | §4 item 2 (the cuckoo filter), TS-16, TS-17 | the leaf's form; negative and null priors |
| D.6.11 | §4 item 9 (six runtime `SearchParams` reads) | behaviour-neutral |
| **D.9** (new) | D.9.1 TT-2 (done), D.9.2 TM-2 (done), D.9.3 TB-1, D.9.4 TM-9, D.9.5 TT-1, D.9.6 TM-7 | instruments (§13b) |
| E.1 | TT-1, D.9.5's table re-read at the release head | instrument |
| E.3.0 | Triumviratus named as a mechanisms-only donor | disclosure |
| Phase F rules | the seven requirements of §12b | requirements |
| F.0 | §12b, TN-1, TN-3, TN-6, TD-16 (the step shape, Stockfish's networks in October 2026) | |
| F.1 | TB-10 | |
| F.2 | TD-9, TD-10 | |
| F.2.1 | TE-1 | screen |
| F.3 | TD-1–TD-4, TD-7, TD-11, TD-12, TD-15 (the average) | |
| F.4 | TN-9 | instruments |
| F.5 | TN-5, TN-7, TN-8, TD-15 (the permutation), TB-14 (AVX-VNNI), TB-15 | |
| F.6 | TD-8, TM-3, TM-5, TM-8, §4 item 15 (ranges) | registration inputs |
| F.7 | TN-1–TN-6, TE-2, TD-6, TD-13, TD-16 | |
| F.8 | TD-5, TD-6, TD-14 | |
| G.1 | TB-13, TP-6 (TP-4 if D.2 closes NOT DEFICIENT) | |
| G.2 | TB-9, TB-14 (universal binary, per-tier PGO), TA-1, TA-3 | |
| G.3 | TS-2 (long-control warning), TS-19, TM-5 | |
| **H.1** (new) | TM-A; the re-ranking of everything below; Basilisk's opening trigger | triage |
| **H.2** (new) | TS-3, TS-4, TS-5, TS-6, TS-7 (a prior), TS-9, TS-10, TS-11, TS-12 (if experts), §4 item 8 (the minor-each draw) | small search gains |
| **H.3** (new) | TT-3, TT-5–TT-10 | small clock gains |
| **H.4** (new) | TB-4–TB-8, TB-11, TB-12, TB-14 (layout), TB-16, TM-10 | identical-tree speed on the network head |
| none | TS-8 (Rarog's form already), TS-13, TS-13b, TS-18, the rest of TS-18b, TN-6 (refused or already covered); TM-1, TM-4, TM-6, TB-2, TB-3, TP-2, TP-5, TT-4 (checked) | |

Phase H holds Basilisk's split of small gains: H.1 triage, H.2 search,
H.3 time management. It adds H.4, because Rarog has no open speed pass
before the network: C.12 is closed, and F.5 covers the network's inference
only.

## 15. What Basilisk adopted, and what Rarog takes

Read on 2026-11-03 from Basilisk's PLAN after its four Triumviratus commits of
that day. All four are documents and its snapshot; Basilisk has implemented
none of the review in code yet. Its packet is
`D:/code/basilisk/analysis/triumviratus_review_2026-11-03.md`; its ledger
imports are BAS-X37 to BAS-X40.

Basilisk placed more of the review in early leaves because its search
programme is still open (its B.3 tune is running). It uses B.4 quiescence, B.5
root and line growth, B.6.1 donor reconciliation, B.7.2 speed pass, B.15 board
review, C.0 residuals, D.1 time management and D.3.2 protocol. Rarog closed
those programmes: Phase B on 2026-10-05, Phase C on 2026-10-22, D.1 on
2026-10-26. So their items here go to an open leaf or wait in Phase H.

| Basilisk leaf | What it took | Rarog |
|---|---|---|
| B.16, the instruments (TT-2, TM-9, TT-1, TM-7) | four instruments before its next cluster | **Adopted as D.9**, before D.5, with two more: the schedule read and the search-path perft. The wall-clock and schedule reads are done |
| B.3.3 | the schedule check on its live 0.5.0 block | D.9.2, done: the same pinned executable |
| B.4, quiescence (TS-13, TS-14) | its quiet-check card | Rarog's quiescence has no quiet checks; TS-14 goes to D.5.2 |
| B.5, root, aspiration, line growth (TS-1, TS-15) | TS-1 as a third form | D.5.7 and D.5.13 |
| B.6.1, donor reconciliation (TS-16, TS-17) | priors | D.5.18 |
| B.15, board review and rebuild (TB-1 to TB-10) | B.15.1, a board-contract audit porting Rarog's board-v2 corpus; B.15.2, a component review | **Not adopted.** B.15.1 ports the contract work Rarog did in 2026-09, which is a standing contract here. Rarog's board is at measured parity, and C.12 read the speed items. TB-1 is D.9.3, TB-10 is in F.1, and the rest is H.4 (§2) |
| B.7.2, speed pass (TB-8, TB-11 to TB-16, TM-10) | read before the network | H.4: C.12 is closed, so they are read on the network head |
| C.0 (TE-1, TE-2, TM-5) | residuals on its evaluation | TE-1 goes to F.2.1, TE-2 to F.7 |
| D.1 (TT-3 to TT-6), reading B.16's loss table first | its time management is still open | Rarog's D.1 is closed; H.3 reads D.9.5 first |
| D.2 (TP-1 to TP-5) | TT16 as a candidate for its TT incoherence | D.2 and D.5.12. Rarog's shared table already verifies by XOR |
| D.3.2, protocol (TR-1 to TR-5) | a test for each defect | Rarog's D.3.1–D.3.8 closed the comparable defects. The paths the checks read correct but untested go to D.4.17 as tests; the overflow is D.4.23 |
| F.0 to F.8 | the network path, leaf by leaf | **Adopted in full**: the lessons block and inputs F.0–F.8 |
| G.1, G.2, G.3 | TP-6; TB-9 and TA; TS-19 | the same |
| H.1 to H.3 | the small gains, and an opening trigger for H.1 | the same, plus H.4; the trigger adopted |

**Also not taken:** the snapshot is in ignored storage here, not tracked
(maintainer decision 2026-11-03, §1).
