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
Write-Host ("      NMP sampled cut / attempt   : {0,7:N2} %   verification pass/fail {1:N0}/{2:N0}" -f `
    (Ratio (Value 'nmp_sample_cut') (Value 'nmp_attempt')), `
    (Value 'nmp_verify_pass'), (Value 'nmp_verify_fail'))
Write-Host ("      best move first in picker   : {0,7:N2} %" -f `
    (Ratio (Value 'best_rank_1') $bestSamples))
Write-Host ("      pruning overlap / candidates: {0,7:N2} %   check exemptions {1:N0}" -f `
    (Ratio (Value 'prune_shadow_overlap_two_plus') (Value 'prune_shadow_moves')), `
    (Value 'prune_shadow_check_exempt'))
Write-Host ("      correction slot collisions : {0:N0} of {1:N0} sampled observations; near rail {2:N0}" -f `
    (Value 'corr_slot_collision'), `
    ((Value 'corr_slot_first') + (Value 'corr_slot_repeat') + (Value 'corr_slot_collision')), `
    (Value 'corr_slot_near_saturation'))
Write-Host ("      root iterations / best-move changes : {0:N0} / {1:N0}" -f `
    $rootIterations, (Value 'root_best_changes'))
Write-Host ""

# B.1 removed the root-confidence model, TT provenance, the 4.2b contradiction
# shadow, the 4.3 refinement shadow and the 4.4a switch sizing together with
# their counters (analysis/search_programme_2026-09-13.md section 6.4).

# 4.5: IS a capture-caused residual actually noisier? Capture weighting
# assumes it is; the all-or-nothing guard was rejected and removed.
$rcN = Value 'corr_resid_capture_n'
$rqN = Value 'corr_resid_quiet_n'
if (($rcN + $rqN) -gt 0) {
    $rcMean = if ($rcN -gt 0) { (Value 'corr_resid_capture_sum') / $rcN } else { 0 }
    $rqMean = if ($rqN -gt 0) { (Value 'corr_resid_quiet_sum') / $rqN } else { 0 }
    Write-Host "  4.5 CORRECTION RESIDUAL BY ATTRIBUTION (exact)"
    Write-Host ("      capture-caused : {0,10:N0} updates, mean |residual| {1,7:N1} cp" -f $rcN, $rcMean)
    Write-Host ("      quiet-caused   : {0,10:N0} updates, mean |residual| {1,7:N1} cp" -f $rqN, $rqMean)
    Write-Host ("      capture share  : {0,7:N2} %   ratio of means {1:N3}" -f `
        (Ratio $rcN ($rcN + $rqN)), $(if ($rqMean -gt 0) { $rcMean / $rqMean } else { 0 }))
    Write-Host "      (ratio near 1.0 means the down-weighting premise is unsupported)"
    Write-Host ""
}

# 4.5d: does a halfmove-clock context carry usable signal? PLAN 4.5 permits a
# new correction context only where held-out UNIQUE signal is shown, so the
# POPULATION matters as much as the mean - a context nothing lands in cannot
# be learned however distinct its residuals look.
$hmTotal = 0
foreach ($b in @("low","mid","high")) { $hmTotal += Value "corr_resid_hm_${b}_n" }
if ($hmTotal -gt 0) {
    Write-Host "  4.5d CORRECTION RESIDUAL BY HALFMOVE CLOCK (exact)"
    foreach ($b in @(@("low","0-19"), @("mid","20-49"), @("high","50+"))) {
        $n = Value "corr_resid_hm_$($b[0])_n"
        $sum = Value "corr_resid_hm_$($b[0])_sum"
        $mean = if ($n -gt 0) { $sum / $n } else { 0 }
        Write-Host ("      clock {0,-6} : {1,9:N0} updates ({2,6:N2} %)  mean |residual| {3,7:N1} cp" -f $b[1], $n, (Ratio $n $hmTotal), $mean)
    }
    $updates = Value "correction_updates"
    if ([Math]::Abs($hmTotal - $updates) -lt 0.5) {
        Write-Host ("      reconciles with correction_updates: {0:N0} OK" -f $updates)
    } else {
        Write-Host ("      *** MISMATCH: buckets {0:N0} vs correction_updates {1:N0} ***" -f $hmTotal, $updates) -ForegroundColor Red
    }
    Write-Host "      check/evasion context is structurally unreachable: correction trains"
    Write-Host "      only where static_eval != VALUE_NONE, i.e. only when NOT in check."
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
