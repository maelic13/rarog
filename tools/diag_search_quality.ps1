<#
.SYNOPSIS
    Phase 4.1 interaction map plus the legacy first-move-cutoff and LMR
    readouts over the deterministic `bench` suite.

.DESCRIPTION
    Runs `bench <depth>` on a `--features diag` build and aggregates the
    per-position `info string diag <name> <value>` dumps. Exact legacy event
    counters and deterministic 1/1024 Phase-4 samples are reported separately.

    WHY THESE TWO. 10.0 measured Rarog's eval at parity with Basilisk 1.9.1
    (paired Texel loss -0.0003 +/- 0.0012 over 8,000 quiet positions) and its
    NPS as equal, yet Rarog plays 38-55 Elo weaker at one thread at BOTH time
    controls. So the deficit is search accuracy, and it splits two ways that
    imply opposite fixes:

      * ORDERING  - first-move cutoff rate = cutoff_first_move
                    / (cutoff_quiet + cutoff_capture). The share of beta
                    cutoffs delivered by the node's FIRST move. Healthy engines
                    sit ~90%+; materially below implicates move ordering, in
                    which case re-tuning the selectivity surface (10.4.6) is
                    aimed at the wrong half of the problem.
      * DEPTH     - over-reduction ratio = lmr_research / lmr_applied. The
                    share of LMR reductions that had to be re-searched at full
                    depth, i.e. reductions the search itself disagreed with.

    Absolute measurement only. Basilisk's internal counters are off limits
    (its tree is read only), so this can never be a head-to-head - it is read
    against public engines' published figures and against Rarog's own history.

    The suite is `bench`: 40 fixed positions, single threaded, deterministic,
    and the same corpus Basilisk and Hydra use, so a reading is reproducible
    and comparable across Rarog revisions. Each position is its own search, so
    the engine emits one dump per position and this script sums them.

.PARAMETER Exe
    Path to a diag-enabled binary. Build one with:
        cargo build --release --features diag

.PARAMETER Depth
    Bench depth. Keep 13 (the project default) for comparability.

.PARAMETER Csv
    Optional path to append one machine-readable row per run, so a later
    revision (e.g. post-10.2.5) can be compared against this baseline.

.EXAMPLE
    cargo build --release --features diag
    ./tools/diag_search_quality.ps1
#>
[CmdletBinding()]
param(
    [string]$Exe = "$PSScriptRoot\..\target\release\rarog.exe",
    [int]$Depth = 13,
    [int]$TimeoutSec = 900,
    [string]$Csv = ""
)

$ErrorActionPreference = "Stop"
. "$PSScriptRoot\uci_probe.ps1"

if (-not (Test-Path $Exe)) {
    throw "Engine not found: $Exe. Build it with: cargo build --release --features diag"
}
$Exe = (Resolve-Path $Exe).Path

# Live process, never `bench | quit` - bench is queued asynchronously and a
# piped `quit` tears the engine down before the suite finishes (it emits the
# banner and nothing else, which reads exactly like a broken build).
$p = Start-Engine $Exe
Send-Line $p "uci"
[void](Read-Until $p "^uciok" 20)
Send-Line $p "bench $Depth"
$out = Read-Until $p "^Nodes/second" $TimeoutSec
Send-Line $p "quit"
Start-Sleep -Milliseconds 200
if (-not $p.HasExited) { $p.Kill() }

$totals = @{}
$dumps = 0
foreach ($line in $out) {
    $m = [regex]::Match($line, 'diag (\w+) (\d+)')
    if (-not $m.Success) { continue }
    $name = $m.Groups[1].Value
    if ($name -eq 'nodes') { $dumps++ }
    $totals[$name] = [double]$totals[$name] + [double]$m.Groups[2].Value
}

if ($dumps -eq 0) {
    $head = ($out | Select-Object -First 5) -join "`n"
    throw ("No diag dumps in the output - the binary is almost certainly built " +
           "WITHOUT --features diag. Rebuild with:`n" +
           "  cargo build --release --features diag`nFirst lines were:`n$head")
}

$fingerprint = ($out | Select-String '^Nodes searched\s*:\s*(\d+)').Matches.Groups[1].Value
$ebf = ($out | Select-String '^Geomean EBF\s*:\s*([\d.]+)').Matches.Groups[1].Value
$nps = ($out | Select-String '^Nodes/second\s*:\s*(\d+)').Matches.Groups[1].Value

function Ratio($num, $den) {
    if ($den -le 0) { return [double]::NaN }
    return 100.0 * $num / $den
}

function Value([string]$name) {
    if ($totals.ContainsKey($name)) { return [double]$totals[$name] }
    return 0.0
}

$cutoffs = $totals['cutoff_quiet'] + $totals['cutoff_capture']
$firstRate = Ratio $totals['cutoff_first_move'] $cutoffs
$overRed = Ratio $totals['lmr_research'] $totals['lmr_applied']
$cutoffShare = Ratio $cutoffs $totals['nodes']

Write-Host ""
Write-Host "======================================================="
Write-Host "  Phase 4.1 interaction map - bench $Depth, 1 thread"
Write-Host "  exe:         $(Split-Path $Exe -Leaf)"
Write-Host "  positions:   $dumps    fingerprint: $fingerprint    geomean EBF: $ebf    NPS: $nps"
Write-Host "======================================================="
Write-Host ""
Write-Host ("  FIRST-MOVE CUTOFF RATE : {0,7:N2} %   ({1:N0} of {2:N0} cutoffs)" -f `
    $firstRate, $totals['cutoff_first_move'], $cutoffs)
Write-Host ("      quiet cutoffs      : {0,12:N0}" -f $totals['cutoff_quiet'])
Write-Host ("      capture cutoffs    : {0,12:N0}" -f $totals['cutoff_capture'])
Write-Host ("      cutoff nodes/nodes : {0,7:N2} %" -f $cutoffShare)
Write-Host ""
Write-Host ("  LMR OVER-REDUCTION     : {0,7:N2} %   ({1:N0} re-searches of {2:N0} reductions)" -f `
    $overRed, $totals['lmr_research'], $totals['lmr_applied'])
Write-Host ""
Write-Host "  SAMPLED INTERACTION MAP (deterministic 1/1024 nodes)"
$ttSamples = (Value 'tt_sample_hit') + (Value 'tt_sample_miss')
$ttCuts = (Value 'tt_cut_exact') + (Value 'tt_cut_lower') + (Value 'tt_cut_upper')
$bestSamples = (Value 'best_rank_1') + (Value 'best_rank_2_3') + `
    (Value 'best_rank_4_7') + (Value 'best_rank_8_plus')
$rootIterations = Value 'root_iterations'
Write-Host ("      TT hit / sampled main nodes : {0,7:N2} %   usable cutoffs {1:N0}; contradictions {2:N0}" -f `
    (Ratio (Value 'tt_sample_hit') $ttSamples), $ttCuts, (Value 'tt_bound_contradicts_window'))
Write-Host ("      qsearch producers           : stand-pat {0:N0}; searched qmove {1:N0}; tail exact/upper {2:N0}/{3:N0}" -f `
    (Value 'q_stand_pat_store'), (Value 'q_move_store'), `
    (Value 'q_tail_exact_store'), (Value 'q_tail_upper_store'))
Write-Host ("      NMP sampled cut / attempt   : {0,7:N2} %   verification pass/fail {1:N0}/{2:N0}; nested {3:N0}" -f `
    (Ratio (Value 'nmp_sample_cut') (Value 'nmp_attempt')), `
    (Value 'nmp_verify_pass'), (Value 'nmp_verify_fail'), (Value 'nmp_nested_attempt'))
Write-Host ("      best move first in picker   : {0,7:N2} %   reduced winners {1:N0}" -f `
    (Ratio (Value 'best_rank_1') $bestSamples), (Value 'best_was_reduced'))
Write-Host ("      pruning overlap / candidates: {0,7:N2} %   check exemptions {1:N0}" -f `
    (Ratio (Value 'prune_shadow_overlap_two_plus') (Value 'prune_shadow_moves')), `
    (Value 'prune_shadow_check_exempt'))
Write-Host ("      correction slot collisions : {0:N0} of {1:N0} sampled observations; near rail {2:N0}" -f `
    (Value 'correction_slot_collision'), `
    ((Value 'correction_slot_first') + (Value 'correction_slot_repeat') + (Value 'correction_slot_collision')), `
    (Value 'correction_slot_near_saturation'))
Write-Host ("      root mean gap / effort      : {0:N2} cp / {1:N2} % over {2:N0} iterations" -f `
    ((Value 'root_gap_sum') / [Math]::Max(1.0, $rootIterations)), `
    ((Value 'root_effort_ppm_sum') / [Math]::Max(1.0, $rootIterations) / 10000.0), `
    $rootIterations)
Write-Host ""

# 4.2 producer census. EXACT, not sampled - this block is the reason the
# sampled producer lines above must not be read as shares. Sampled counters at
# different node classes do not share a denominator, which understated ProbCut
# by 2.4x in the first 4.2 reading (RAR-S22).
$kinds = [ordered]@{
    'full'             = 'store_kind_full'
    'verified reduced' = 'store_kind_verified_reduced'
    'qsearch move'     = 'store_kind_qsearch_move'
    'qsearch tail'     = 'store_kind_qsearch_tail'
    'stand pat'        = 'store_kind_stand_pat'
    'ProbCut'          = 'store_kind_probcut'
    'tablebase'        = 'store_kind_tablebase'
}
$kindTotal = 0.0
foreach ($counter in $kinds.Values) { $kindTotal += Value $counter }
if ($kindTotal -gt 0) {
    Write-Host "  TT PRODUCER CENSUS (exact, by declared OutcomeKind)"
    foreach ($label in $kinds.Keys) {
        $count = Value $kinds[$label]
        Write-Host ("      {0,-16} {1,14:N0}   {2,6:N2} %" -f $label, $count, (Ratio $count $kindTotal))
    }
    $horizon = (Value 'store_kind_qsearch_move') + (Value 'store_kind_qsearch_tail') +
               (Value 'store_kind_stand_pat')
    Write-Host ("      {0,-16} {1,14:N0}   {2,6:N2} %" -f '-> depth-0 total', $horizon,
        (Ratio $horizon $kindTotal))
    # The census must account for every store. `fresh + same_key` is counted on
    # a different code path, so a mismatch means a store site bypassed the
    # census or a kind is being miscounted into a neighbouring bucket.
    $storeTotal = (Value 'tt_store_fresh') + (Value 'tt_store_same_key')
    if ([Math]::Abs($kindTotal - $storeTotal) -lt 0.5) {
        Write-Host ("      reconciles with tt_store_fresh + same_key: {0:N0} OK" -f $storeTotal)
    } else {
        Write-Host ("      *** CENSUS MISMATCH: kinds {0:N0} vs stores {1:N0} ***" -f `
            $kindTotal, $storeTotal) -ForegroundColor Red
    }

    # 4.3 provenance hazards.
    #
    # ⚠ DENOMINATORS. `store_kind_*` counts ATTEMPTS (it runs before the backend
    # dispatch, hence the reconciliation above); the hazard counters run after
    # the depth-preservation `return` and count COMMITTED stores. Every rate
    # below therefore uses `store_committed_*`, never the census. Mixing them
    # was the original error in these figures and it biased them LOW.
    $skipped = Value 'store_skipped_depth_rule'
    $spCommitted = Value 'store_committed_stand_pat'
    $qmvCommitted = Value 'store_committed_qsearch_move'
    $horizonCommitted = Value 'store_committed_horizon'
    $inheritedSp = Value 'tt_move_inherited_stand_pat'
    Write-Host ""
    Write-Host "  4.3 PROVENANCE HAZARDS (exact, committed-store denominators)"
    Write-Host ("      attempted / skipped by depth rule / committed : {0:N0} / {1:N0} / {2:N0}" -f `
        $kindTotal, $skipped, ($kindTotal - $skipped))
    # attempted - skipped == committed must hold on both backends. If it does
    # not, a store path is bypassing one of the two counters.
    $horizonAttempted = (Value 'store_kind_qsearch_move') + (Value 'store_kind_qsearch_tail') +
                        (Value 'store_kind_stand_pat')
    if ($horizonCommitted -gt $horizonAttempted) {
        Write-Host "      *** COMMITTED EXCEEDS ATTEMPTED - counter placement is wrong ***" -ForegroundColor Red
    }
    Write-Host ("      stand pat: committed {0:N0} of {1:N0} attempted  ({2:N2} % skipped)" -f `
        $spCommitted, (Value 'store_kind_stand_pat'), `
        (100.0 - (Ratio $spCommitted (Value 'store_kind_stand_pat'))))
    Write-Host ("      stand-pat stores that inherited a move : {0,10:N0}   {1,6:N2} % of COMMITTED stand pat" -f `
        $inheritedSp, (Ratio $inheritedSp $spCommitted))
    Write-Host ("      all moveless stores that inherited     : {0,10:N0}" -f (Value 'tt_move_inherited'))
    Write-Host ("      horizon store overwrote deeper entry   : {0,10:N0}   {1,6:N2} % of committed horizon" -f `
        (Value 'tt_horizon_overwrote_searched'), (Ratio (Value 'tt_horizon_overwrote_searched') $horizonCommitted))
    # A shape test for "searched qmove" is `depth 0 + Lower + has a move`. Stand
    # pat with an inherited move satisfies it too, so this is the false-positive
    # rate a provenance-free 4.3 inference would carry. Both terms are committed.
    Write-Host ("      => shape test 'depth 0 + Lower + move' leak rate : {0,6:N2} %   ({1:N0} of {2:N0})" -f `
        (Ratio $inheritedSp ($qmvCommitted + $inheritedSp)), $inheritedSp, ($qmvCommitted + $inheritedSp))
    Write-Host ""
}

# 4.2b shadow test. A contradicting entry cannot cut off (unit-tested in
# evidence.rs), so everything here is a NON-cutoff consumer admitting evidence
# that told this node nothing. Sampled, so read shares, not absolute volumes.
$contradictHits = Value 'contradict_hits'
if ($contradictHits -gt 0) {
    Write-Host "  4.2b CONTRADICTING INEXACT BOUNDS (sampled; nothing branches on these)"
    Write-Host ("      contradicting hits          : {0,7:N2} %   ({1:N0} of {2:N0} sampled hits)" -f `
        (Ratio $contradictHits (Value 'tt_sample_hit')), $contradictHits, (Value 'tt_sample_hit'))

    $refined = Value 'contradict_refined_eval'
    $meanDelta = if ($refined -gt 0) { (Value 'contradict_refine_delta_sum') / $refined } else { 0 }
    Write-Host ("      moved eval_for_pruning      : {0,7:N2} %   ({1:N0} of {2:N0}); mean shift {3:N1} cp" -f `
        (Ratio $refined $contradictHits), $refined, $contradictHits, $meanDelta)
    # Slack = ev.depth - EvalPruneTtMinDepth. A penalty of P plies blocks every
    # bucket below P, so this row IS the answer for each candidate P.
    Write-Host ("        slack 0 / 1 / 2-3 / 4-7 / 8+ : {0:N0} / {1:N0} / {2:N0} / {3:N0} / {4:N0}" -f `
        (Value 'contradict_refine_slack_0'), (Value 'contradict_refine_slack_1'), `
        (Value 'contradict_refine_slack_2_3'), (Value 'contradict_refine_slack_4_7'), `
        (Value 'contradict_refine_slack_8_plus'))

    $csa = Value 'contradict_singular_attempt'
    # Extensions and multi-cuts are counted at separate sites (the multi-cut arm
    # returns), so total tree effect is their sum.
    $csChanged = Value 'contradict_singular_changed_depth'
    $csMulticut = Value 'contradict_singular_multicut'
    Write-Host ("      seeded a singular window    : {0:N0} of {1:N0} attempts" -f `
        $csa, (Value 'singular_attempt'))
    Write-Host ("        -> changed depth {0:N0}, multi-cut {1:N0}, total effect {2:N0} ({3:N1} % of seeded)" -f `
        $csChanged, $csMulticut, ($csChanged + $csMulticut), (Ratio ($csChanged + $csMulticut) $csa))
    Write-Host ("      suppressed IIR              : {0:N0}" -f (Value 'contradict_iir_suppressed'))

    # 4.3: is TT eval refinement self-cancelling? Two arms of
    # EvalPruneTtMinDepth measured ~0 Elo while moving 15-44% of the tree; the
    # margins-absorb-it and helps-as-often-as-it-hurts explanations imply
    # opposite fixes.
    $flipNodes = Value 'refine_flip_nodes'
    if ($flipNodes -gt 0) {
        Write-Host ""
        Write-Host "  4.3 IS EVAL REFINEMENT SELF-CANCELLING? (sampled)"
        Write-Host ("      nodes where refinement moved the eval : {0:N0}" -f $flipNodes)
        # Unbiased half: counted before any consumer can return.
        $onTotal = 0; $offTotal = 0
        foreach ($c in @('rfp','razor','nmp')) {
            $on = Value "refine_flip_${c}_on"; $off = Value "refine_flip_${c}_off"
            $onTotal += $on; $offTotal += $off
            Write-Host ("        {0,-6} caused {1,6:N0} / prevented {2,6:N0}   net {3,7:N0}" -f `
                $c, $on, $off, ($on - $off))
        }
        Write-Host ("        TOTAL  caused {0,6:N0} / prevented {1,6:N0}   net {2,7:N0}  ({3:N1} % of moved nodes flipped a decision)" -f `
            $onTotal, $offTotal, ($onTotal - $offTotal), (Ratio ($onTotal + $offTotal) $flipNodes))
        # Biased half - a pruned node never reaches the tail, so the cases where
        # refinement mattered most are absent. Read with the flip counts above.
        $rn = Value 'refine_report_nodes'
        if ($rn -gt 0) {
            $closer = Value 'refine_report_closer'; $farther = Value 'refine_report_farther'
            Write-Host ("      agreed with the reported score : closer {0:N0} / farther {1:N0} of {2:N0}  ({3:N1} % closer)" -f `
                $closer, $farther, $rn, (Ratio $closer $rn))
            $gainSum = Value 'refine_report_gain_sum'
            $lossSum = Value 'refine_report_loss_sum'
            Write-Host ("        mean cp gained when closer {0,7:N1} / lost when farther {1,7:N1}   net {2,8:N0} cp" -f `
                ($gainSum / [Math]::Max(1, $closer)), `
                ($lossSum / [Math]::Max(1, $farther)), `
                ($gainSum - $lossSum))
            Write-Host "        (biased: excludes every node refinement pruned - see the flip counts)"
        }
    }

    # THE decision row. If these two rates are close, a depth/confidence penalty
    # belongs on the SCORE consumers only and must leave ordering and IIR alone.
    $cPresent = Value 'contradict_move_present'
    $aPresent = Value 'agree_move_present'
    Write-Host ("      TT move best - contradicting: {0,7:N2} %   ({1:N0} of {2:N0})" -f `
        (Ratio (Value 'contradict_move_was_best') $cPresent), (Value 'contradict_move_was_best'), $cPresent)
    Write-Host ("      TT move best - agreeing     : {0,7:N2} %   ({1:N0} of {2:N0})" -f `
        (Ratio (Value 'agree_move_was_best') $aPresent), (Value 'agree_move_was_best'), $aPresent)
    Write-Host ""
}
Write-Host "  Raw counters:"
foreach ($k in ($totals.Keys | Sort-Object)) {
    Write-Host ("      {0,-28} {1,14:N0}" -f $k, $totals[$k])
}
Write-Host ""

if ($Csv) {
    $repoSha = (git -C "$PSScriptRoot\.." rev-parse --short HEAD 2>$null)
    if (-not $repoSha) { $repoSha = "n/a" } else { $repoSha = $repoSha.Trim() }
    if (-not (Test-Path $Csv)) {
        "utc,sha,depth,fingerprint,first_move_cutoff_pct,over_reduction_pct,cutoffs,lmr_applied" |
            Set-Content -Path $Csv -Encoding utf8
    }
    ("{0},{1},{2},{3},{4:N4},{5:N4},{6:N0},{7:N0}" -f `
        (Get-Date).ToUniversalTime().ToString('u'), $repoSha, $Depth, $fingerprint,
        $firstRate, $overRed, $cutoffs, $totals['lmr_applied']) |
        Add-Content -Path $Csv -Encoding utf8
    Write-Host "  Appended to $Csv"
    Write-Host ""
}
