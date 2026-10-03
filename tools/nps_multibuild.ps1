# Multi-build PGO A/B: pools N independent PGO builds per arm.
#
# Superseded for acceptance reads by tools/nps_read.py (the two-step read of
# PROCESS, RAR-P32): it archives every run and reports a per-build interval.
# This script's bootstrap interval resamples readings, not builds, and is not
# the uncertainty of a source change. Kept for comparison with earlier rows.
#
# A single PGO build per arm carries a fixed per-binary offset of ~0.4% (a null
# pair of identical source measured -0.36%, CI -0.75..-0.06). Pooling several
# independent builds per arm averages that profile luck out, which is the only
# way to resolve a sub-1% effect in the shipped configuration.
param(
    [int]$Cycles = 10,
    [int]$Repeats = 3,
    [Parameter(Mandatory = $true)][string[]]$BaseSet,
    [Parameter(Mandatory = $true)][string[]]$CandSet,
    [string]$Label = "cand vs base"
)

function Get-Nps([string]$exe, [int]$repeats) {
    $out = "bench 13 $repeats" | & $exe 2>&1
    $line = $out | Select-String -Pattern "^Nodes/second" | Select-Object -Last 1
    if ($line -match "Nodes/second\s*:\s*(\d+)") { return [double]$Matches[1] }
    throw "no NPS from $exe"
}
function Get-Median($a) {
    $s = $a | Sort-Object; $n = $s.Count
    if ($n % 2) { return $s[[int]($n / 2)] }
    return ($s[$n / 2 - 1] + $s[$n / 2]) / 2
}

"== $Label =="
"base builds: $($BaseSet -join ', ')"
"cand builds: $($CandSet -join ', ')"

foreach ($e in $BaseSet + $CandSet) { $null = Get-Nps $e $Repeats }  # warm-up

$perBuild = @{}
foreach ($e in $BaseSet + $CandSet) { $perBuild[$e] = @() }

for ($c = 0; $c -lt $Cycles; $c++) {
    # Interleave one reading of every build per cycle, alternating direction so
    # no build sits in the same slot every time.
    $order = if ($c % 2 -eq 0) { $BaseSet + $CandSet } else { ($CandSet + $BaseSet) }
    foreach ($e in $order) { $perBuild[$e] += Get-Nps $e $Repeats }
}

""
"per-build medians:"
$baseVals = @(); $candVals = @()
foreach ($e in $BaseSet) {
    $m = Get-Median $perBuild[$e]; $baseVals += $perBuild[$e]
    "  BASE {0,-46} {1,10:N0}" -f (Split-Path $e -Leaf), $m
}
foreach ($e in $CandSet) {
    $m = Get-Median $perBuild[$e]; $candVals += $perBuild[$e]
    "  CAND {0,-46} {1,10:N0}" -f (Split-Path $e -Leaf), $m
}

$bMed = Get-Median $baseVals; $cMed = Get-Median $candVals
$bMax = ($baseVals | Measure-Object -Maximum).Maximum
$cMax = ($candVals | Measure-Object -Maximum).Maximum

$rng = [System.Random]::new(9876)
$boot = @()
for ($k = 0; $k -lt 2000; $k++) {
    $ra = @(); $rb = @()
    for ($j = 0; $j -lt $baseVals.Count; $j++) { $ra += $baseVals[$rng.Next(0, $baseVals.Count)] }
    for ($j = 0; $j -lt $candVals.Count; $j++) { $rb += $candVals[$rng.Next(0, $candVals.Count)] }
    $boot += ((Get-Median $rb) - (Get-Median $ra)) / (Get-Median $ra) * 100.0
}
$bs = $boot | Sort-Object
""
"RESULT [$Label]  {0} cycles, {1} base builds vs {2} cand builds" -f $Cycles, $BaseSet.Count, $CandSet.Count
"  pooled median  base {0,10:N0}  cand {1,10:N0}   delta {2,6:N2}%" -f $bMed, $cMed, (($cMed - $bMed) / $bMed * 100)
"  pooled best-of base {0,10:N0}  cand {1,10:N0}   delta {2,6:N2}%" -f $bMax, $cMax, (($cMax - $bMax) / $bMax * 100)
"  median delta 95% bootstrap CI: {0,6:N2}% .. {1,6:N2}%" -f $bs[[int](0.025 * $bs.Count)], $bs[[int](0.975 * $bs.Count)]
