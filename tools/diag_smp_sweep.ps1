. "$PSScriptRoot\uci_probe.ps1"

# SMP diagnostic sweep. Same two positions as the scaling run so the two
# measurements are comparable. 3 reps because Lazy SMP has ~2 iterations of
# rep-to-rep spread and a single reading already produced one false conclusion.
$EXE = "$PSScriptRoot\..\target\release\rarog.exe"  # build with: cargo build --release --features diag
$POS = @(
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "r1bq1rk1/pp2ppbp/2np1np1/8/2BNP3/2N1BP2/PPPQ2PP/R3K2R w KQ - 0 1"
)
$REPS = 3
$MOVETIME = 3000

function Get-Diag([int]$threads, [string]$fen) {
    $p = Start-Engine $EXE
    Send-Line $p "uci"; [void](Read-Until $p "^uciok" 20)
    Send-Line $p "setoption name Hash value 256"
    Send-Line $p "setoption name Threads value $threads"
    Send-Line $p "ucinewgame"; Send-Line $p "isready"; [void](Read-Until $p "^readyok" 40)
    Send-Line $p "position fen $fen"
    Send-Line $p "go movetime $MOVETIME"
    $out = Read-Until $p "^bestmove" 90
    Send-Line $p "quit"; Start-Sleep -Milliseconds 150
    if (-not $p.HasExited) { $p.Kill() }

    $d = @{}
    $depths = @()
    foreach ($line in $out) {
        $m = [regex]::Match($line, 'diag (\w+) (\d+)')
        if (-not $m.Success) { continue }
        $name = $m.Groups[1].Value
        $val = [double]$m.Groups[2].Value
        if ($name -match '^thread_depth_(\d+)$') { $depths += $val } else { $d[$name] = $val }
    }
    $d['depths'] = $depths
    $d['dumps'] = ($out | Select-String 'diag nodes ' | Measure-Object).Count
    return $d
}

function Med($a) {
    $s = @($a | Sort-Object); $n = $s.Count
    if ($n -eq 0) { return 0 }
    if ($n % 2) { return $s[[int]($n / 2)] }
    return ($s[$n / 2 - 1] + $s[$n / 2]) / 2
}

"{0,-3} {1,8} {2,9} {3,10} {4,9}  {5}" -f 'T', 'hit%', 'samekey%', 'asp/thread', 'nodes', 'depths (median per thread)'
foreach ($t in 1, 2, 4, 8, 16) {
    $hits = @(); $sk = @(); $asp = @(); $nodes = @(); $depthSets = @(); $dumps = @()
    $cutPct = @(); $shallowPct = @(); $deficit = @(); $pvPct = @(); $windowPct = @()
    foreach ($fen in $POS) {
        for ($r = 0; $r -lt $REPS; $r++) {
            $d = Get-Diag $t $fen
            if (-not $d.ContainsKey('main_tt_probes')) { continue }
            $hits += 100.0 * $d['main_tt_hits'] / [Math]::Max(1, $d['main_tt_probes'])
            $sk += 100.0 * $d['tt_store_same_key'] / [Math]::Max(1, $d['tt_store_same_key'] + $d['tt_store_fresh'])
            $asp += ($d['asp_fail_high'] + $d['asp_fail_low']) / $t
            $nodes += $d['nodes']
            $depthSets += , $d['depths']
            $dumps += $d['dumps']
            # What happens to a sampled TT HIT, by cause. The shares must
            # be read against `tt_sample_hit`, never against probes: the
            # question is what a hit is worth, not how often one occurs.
            $sh = [Math]::Max(1, $d['tt_sample_hit'])
            $cuts = $d['tt_cut_exact'] + $d['tt_cut_lower'] + $d['tt_cut_upper']
            $cutPct += 100.0 * $cuts / $sh
            $shallowPct += 100.0 * $d['tt_reject_shallow'] / $sh
            $pvPct += 100.0 * ($d['tt_reject_pv'] + $d['tt_reject_excluded']) / $sh
            $windowPct += 100.0 * $d['tt_bound_not_usable'] / $sh
            $deficit += $d['tt_reject_shallow_deficit'] / [Math]::Max(1, $d['tt_reject_shallow'])
        }
    }
    # median depth per thread slot across all reps
    $perSlot = @()
    for ($i = 0; $i -lt $t; $i++) {
        $vals = @($depthSets | ForEach-Object { if ($_.Count -gt $i) { $_[$i] } })
        $perSlot += [int](Med $vals)
    }
    "{0,-3} {1,8:N1} {2,9:N1} {3,10:N1} {4,9:N0}  {5}   dumps={6}" -f `
        $t, (Med $hits), (Med $sk), (Med $asp), (Med $nodes), ($perSlot -join ','), ((Med $dumps))
    # Breakdown of the same runs: of every sampled TT hit, what share
    # could actually cut, and what share was refused for each distinct reason.
    "    of hits: cut {0,5:N1}%  shallow {1,5:N1}% (short by {2,4:N1} ply)  pv/excl {3,5:N1}%  wrong-window {4,5:N1}%" -f `
        (Med $cutPct), (Med $shallowPct), (Med $deficit), (Med $pvPct), (Med $windowPct)
}

