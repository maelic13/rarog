# Trinity read, 2026-11-23

A reading of https://github.com/ayman803/trinity (commit `03e5c0a`,
2026-11-22) to answer what approach it chose and what Rarog can learn from
it. Trinity is not a donor (`PROCESS.md`, *The independence boundary*):
this note records mechanisms, measured results and calibration points.
Reckless and Stockfish stay Rarog's primary sources, for the network as for
the search (maintainer decision 2026-11-23). PLAN owns what this produced:
Phase F's research inputs and D.16.

Every number below is Trinity's own measurement, read from its
`ESTIMATES.md` (*Changes*, *Lessons so far*, *Ablation audit*, *Loss
forensics*) and `NETWORK.md` (sections 1 to 6 and the merge notes). Its
self-play tests are SPRTs against its previous `main` at 8+0.08 on the UHO
book; its "CCRL-scale" numbers are its own calibrations against rated
engines at 10+0.1, not CCRL's.

## The engine

- Rust, GPL-3.0, written by Claude Code and directed by a non-programmer,
  first commit 2026-09-30. Official CCRL 40/15 rating of release 1.0.0:
  **3522 ± 26** after 262 games (list of 2026-11-21, rank 76 to 78, one
  CPU). Its development head reads about +75 to +100 over 1.0.0 in its own
  tests. Rarog's C head is about 3,407 on the 40/15 scale by the held
  mapping (RAR-M75), not an official rating.
- NNUE from the first day; the hand-written evaluation only bootstrapped the
  first self-play data. The network: 768 inputs × 8 king buckets (mirrored
  by king file, buckets fine near the home corner) → 1024, both
  perspectives, → 8 output buckets by piece count, SCReLU, QA 255, QB 64,
  scale 400, embedded in the binary, AVX2 output layer.
- Trained with bullet from random weights with a factoriser, on 3.65 billion
  positions converted from the Leela-derived binpacks the Stockfish
  community published. Filter: ply ≥ 16, side to move not in check, a quiet
  best move, |score| ≤ 10,000. AdamW, learning rate 0.001 cosine to 1%,
  batch 16,384, 6,104 batches a superbatch. 600 superbatches took 5.0
  hours on an RTX 3080. The current net: 900 superbatches, game-result
  weight 0.25, eval scaled ×1.2.
- The search is plain: PVS, aspiration, null move, reverse futility,
  futility, late-move pruning, SEE pruning, singular extension with
  multi-cut and a negative extension, IIR, check extension, two killers, a
  counter move, two continuation histories, pawn and non-pawn correction,
  an LMR term that reduces less where the correction is large, simple time
  management, Lazy SMP. No ProbCut, no history pruning. Copy-make board.
- Its own summary: search ideas borrowed from stronger engines gave about
  zero, data and networks almost everything. Its two largest search gains
  were removals of its own over-pruning: razoring (+58.3 ± 6.9) and
  quiescence futility (+59.8 ± 6.8), both cutting quiet tactics the
  evaluation could not see. Neither is evidence about Rarog's kernel.

## Network lessons, with their numbers

1. **Run-to-run spread is large.** Three runs of one recipe that differed
   only in the weight seed (and so the start) scored 0 (the reference),
   −9.67 ± 4.92 and −0.23 ± 4.98 against the first, about 6,000 games
   each: a sample SD of about 5.5 Elo. One comparison read as one change
   (16 king buckets against a control, +4.46 ± 2.87) also changed the
   seed, which Trinity corrected afterwards: its estimate of the buckets'
   effect became about +1 ± 7, not established. For an architecture change
   the input layer cannot share a start; the other layers and the data
   order can.
2. **Held-out loss did not rank nets.** 16 king buckets: held-out loss
   0.27% lower, −2.10 ± 2.85 Elo (18,000 games). L1 2048 against 1024:
   loss 1.3% lower, −29.76 ± 4.16 at 8+0.08 and −19.71 ± 6.61 at 30+0.3,
   with about 31% fewer nodes per second. The loss gate was retired on
   2026-11-08 and kept only as a broken-run check.
3. **Capacity transferred; more training on a converged net did not.** King
   buckets: +19.1 ± 8.5 in self-play, +17 in calibration. 512 → 1024 from
   scratch on the whole file: +80.0 ± 4.3 in self-play, +50 in calibration,
   at a 28% speed cost. Against that: 40 superbatches of unseen data on a
   converged net +6.0 ± 4.1 (calibration about 0); a restart at learning
   rate 0.0005 −13.8 ± 7.9; the same data at a gentle rate −3.8 ± 4.6;
   averages of late checkpoints +0.4 ± 3.9 and −1.3 ± 4.3; a fine-tune on
   the net's largest disagreements with Leela −92 ± 41 (124 games).
4. **Schedule length and the result weight interact.** 900 superbatches
   against 600 at result weight 0.5: +5.65 ± 2.73. Result weight 0.25
   against 0.5 at 900: +5.54 ± 3.80. At 0.25, 900 against 600: −1.0 ± 3.9.
   These are Leela labels; Rarog's own labels need their own read.
5. **Width.** L1 2048 cost 28% to 33% of nodes per second on Trinity's AVX2
   inference and lost at 3.65 billion positions (item 2); 1536 cost 19%.
   Triumviratus's ledger read 2048 at −20% and 1536 at −11.2%.
6. **Quantisation error is measurable and not only random.** On 20,000
   positions from its own games, integer against float network: mean
   |gap| 6.4 cp, 95th percentile 16.6, 1.1% of evals change sign (all near
   zero). First layer alone 3.4 cp, output layer alone 5.4, adding in
   quadrature. The bias differs by output bucket, −4.1 to +5.2 cp, so a
   capture that crosses a bucket boundary moves about 7 cp. 3.4% of output
   weights exceed 1.0, so a finer output scale needs clipping in training.
   The layout check first proved the files matched number for number.
7. **Label strength from a weak generator.** Its first nets, trained on
   about 120 million positions of its own self-play from a days-old engine,
   calibrated about 3,240; the first stage on Leela labels added +169.
   Rarog's generator is about 3,475, so the size does not transfer; the
   direction agrees with Triumviratus's TD-9.
8. **Self-play overstated gains against rated engines** by about 1.1× for
   capacity changes and up to about 10× for more epochs (net #6: +29.1 in
   self-play, about +3 in calibration).

## Measurement lessons

- **Bench speed misread a move-loop change.** A picker change read +5.6%
  nodes per second on Trinity's bench, which priced it about +3. Its half
  that drops the quiet moves pruning has already skipped, the same search
  up to tie order, ran 18% faster (median) on 60 positions from real games
  at 300 ms and gained +18.6 ± 3.9 alone. Trinity's lesson: measure speed on game
  positions. Rarog's speed read is `bench 13 3` (PROCESS); D.7.8.12's
  profile found game play the same shape, which is a profile, not a paired
  delta. D.16 owns the check.
- **A move-agreement audit needs a no-change baseline.** At equal nodes,
  changing only the hash size flipped about 350 of 2,000 moves and netted
  +20 against a 16× reference; most single-rule ablations sat below that.
- **Forecasts were recalibrated from the record:** about one candidate in
  four passed, and from 2026-10-27 ranges were widened 1.5× and pass
  chances scaled by 0.6.
- **Already in Rarog:** the correction-scaled reduction (`lmr_correction`),
  skipping quiet moves once pruning applies (the compact picker), and
  refusing an SPRT between byte-identical binaries (`colosseum.ps1`).

## The testing lane on GitHub Actions

Trinity plays its SPRTs and SPSA on GitHub-hosted runners: five jobs, three
games at a time on each 4-vCPU runner, 8+0.08, fastchess, the result
committed back as a text file; long tests continue from an earlier run's
artifact. Its checks: 9,000 games between identical engines read +0.15, and
king buckets read +16.3 ± 4.4 there against +19.1 ± 8.5 on its PC. About
1,800 games a job in 4 to 5 hours. Its SPSA runs five independent chains of
about 5 hours, averaged, about three runs to a tune.

GitHub's terms and limits as read on 2026-11-23 (docs.github.com: *GitHub
Terms for Additional Products and Features*, *Actions limits*, *GitHub
Actions billing*, *GitHub-hosted runners*): standard hosted runners are
free and unlimited for public repositories (Rarog's is public); a public
repository's `ubuntu-latest` and `windows-latest` runners have 4 vCPU and
16 GB; a job runs at most 6 hours, a workflow run 35 days, a matrix 256
jobs; 20 concurrent jobs on the Free plan, 40 on Pro. Actions may be used
"to develop and test your application(s)", and not for activity whose
burden is disproportionate to its benefit; GitHub may terminate jobs. The
pinned Colosseum CLI 0.5.0 also ships Linux x64 and Windows ARM64 builds.
Using this lane would change AGENTS' measurement rules (the idle host, the
pinned runner, maintainer-owned long runs); it is a maintainer decision,
not a step this read takes.
