# RAR-S72 — 4.6.1 quiet SEE oracle screen

Indexed under *3. Search and selectivity › Closed 4.6 follow-ups* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and reconstruction

**4.6.1 quiet SEE oracle screen.** Rarog ablation arm at `e438ced`, exact options `QuietSeePruneDepth=6,QuietSeePruneCoeff=25`; oracle `AblationMask=0`. Fixed 4,000-game screen, `3+0.03`, 1T, Hash 64, paired UHO, concurrency 14, strength-v2. Candidate/oracle executable SHA-256: `F8710A9A5ABD8E3CF7B708AC096E77F06F79848C8F7E8170F3F1F8A986BDF79A` / `10EB7301E01842C5FF2C70930A0BB01EB079163AEA50FB58871453F717D2A75E`. The option recipe plus current default-off implementation reproduces the arm; require accepted switch-off fingerprint **6,977,070 / EBF 2.466** before using it.

## Result / disposition

**Stopped diagnostic null; candidate remains default-off.** Complete-pair reconstruction gives 326 pairs / **652 games**, score 19.402%, **-247.39 +/- 23.69 Elo**, pentanomial `[138,130,51,7,0]`; three partial rounds excluded. Against the same-condition `G(0) = -250.77 +/- 13.12`, estimated gap closure is only **+3.38 +/- 27.08 Elo**. This is not an SPRT H0.

## Conditional lesson

The 0.20x oracle activation divergence was real, but it did not identify portable headroom. This closes the shallow-selectivity continuation and does not authorize coefficient tuning or a broader Step-13 rewrite.

## Artifact evidence

Local PGN/log/manifest SHA-256: `18DCB60925993AB61EF143E31082333D72763469D2BA3C41E5571DA0D8446030`, `990CA0391816563960B3D92C9E5DF08B90849B6B154F700367D6C84485D05FA5`, `4CF6B238A66D9D57C4891F6FCECC372AE8694FDA94302A3E1D6ACCC98E744FDA`; opening seed `1675297318`; book SHA-256 `7A7F6470615A69C6CF23D565417701D38732876F480AF90D67B42ABADE35644A`
