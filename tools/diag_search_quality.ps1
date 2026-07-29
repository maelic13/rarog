<#
.SYNOPSIS
    10.0(a) search-quality readout: first-move cutoff rate and LMR
    over-reduction ratio over the `bench` suite.

.DESCRIPTION
    Runs `bench <depth>` on a `--features diag` build and aggregates the
    per-position `info string diag <name> <value>` dumps into the two standard
    search-quality ratios.

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

function Ratio($num, $den) {
    if ($den -le 0) { return [double]::NaN }
    return 100.0 * $num / $den
}

$cutoffs = $totals['cutoff_quiet'] + $totals['cutoff_capture']
$firstRate = Ratio $totals['cutoff_first_move'] $cutoffs
$overRed = Ratio $totals['lmr_research'] $totals['lmr_applied']
$cutoffShare = Ratio $cutoffs $totals['nodes']

Write-Host ""
Write-Host "======================================================="
Write-Host "  10.0(a) search-quality readout - bench $Depth, 1 thread"
Write-Host "  exe:         $(Split-Path $Exe -Leaf)"
Write-Host "  positions:   $dumps    fingerprint: $fingerprint    geomean EBF: $ebf"
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
