# The texel delta scale without the rule-50 factor (2026-10-11)

**What changed.** `linear_delta_scale` (`src/eval/trace.rs`, texel builds only)
multiplied each row's feature coefficients by `(199 − min(clock, 100)) / 199`,
a rule-50 factor the evaluator no longer applies: the damping moved into the
search (`CoreEvalRule50Damping`, `src/search/correction.rs`), and the
evaluator's score, which is the fit's base score, ignores the clock. So the fit
compared an undamped score against damped weight deltas, and rows with a high
clock carried too little gradient. Commit `0fe4702` removes the factor and
keeps the endgame scale factors (specialised scales, the opposite-bishop rule,
the KNNK zero), so the factor is now exactly what `scale_endgame` applies. The
production bench is unchanged (12,567,796 / EBF 2.535). `0fe4702`'s test
`delta_scale_follows_the_evaluator_not_the_clock` fails on the old factor.
`eef17d9` moves `ocb_scale_screen.py verify` to the new dump contract;
`--with-rule50` still verifies the dumps RAR-E26 to RAR-E33 wrote.

**What it does to a fit, measured once, nothing refitted.** Recipe:
`analysis/artifacts/coherence-2026-10-11/a1/run.sh`; arms
`analysis/artifacts/c521/rarog-texel-head.exe` (before, `1e3c8a34…`, built for
RAR-E33 from the same engine and tuner source) and the tuner built at `0fe4702`
(after, `7b271ab1…`); `hce-v4-tb/validation.csv` (194,444 rows, `3588865c…`);
K 1.59438 (RAR-E32's fit).

- *The loss report* (`--report-endgames`, the current vector from
  `--write-defaults`, byte-identical in both arms): identical rows, global loss
  0.09385306 in both. Only rows with equal position counts change order
  (the report sorts a hash map by count alone). This is by construction: the
  factor multiplies weight deltas, which are zero at the current vector, so no
  loss at a fixed vector can see it. The change acts on the gradient.
- *The gradient weight it removed* (`a1_gradient_weight.py`): per row,
  `|(y − p)·p·(1 − p)|` times the factor, summed by clock band. The two dumps
  agree on every FEN, label and score, and every old factor equals the new
  one times the rule-50 factor (0 mismatches). Overall the old factor removed
  **1.25%** of this weight, because 94% of validation rows have a clock under
  10. By band: clock 10–19 6.3% (3.9% of rows), 20–39 13.3% (1.3%), 40–59
  24.1% (0.41%), 60–79 34.6% (0.19%), 80–100 43.4% (0.07%). The per-row
  magnitude ignores sign cancellation between rows, so it bounds what any one
  coordinate's gradient lost rather than measuring it.

**Use.** The fitted vector does not change. The next fit (C.6's or C.8's) is
the first under the corrected scale, so a comparison of its coefficients with
the current vector's mixes this change with whatever that unit changes; the
expected effect is small and confined to slow, late-clock positions, which are
mostly endgames. Not Elo.
