#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Measure nodes-to-depth, iteration cost and branching on a fixed corpus.

.DESCRIPTION
    Drives any UCI engine with `go depth N`, using a fresh process at every
    depth and clearing game state between positions. The consecutive-depth
    node ratio is the useful tree-width measure; absolute node counts are not
    comparable between engines that define a node differently. The report also
    retains every position's sequence and differences cumulative totals into
    per-iteration costs. This prevents one endpoint position or a cumulative
    shallow-depth artifact from carrying a diagnosis.

    Hash size, position corpus and depth interval are part of the measurement.
    A report records their hashes so a later comparison cannot splice unlike
    workloads, the error that invalidated Manta's first branching profile.

.EXAMPLE
    pwsh -File tools/branching_profile.ps1 -Engine target/release/rarog.exe `
        -MinDepth 4 -MaxDepth 9 -Hash 64 -OutFile tools/results/branching.json
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Engine,
    [string]$Positions = "$PSScriptRoot\diag\phase4_suite_v1.epd",
    [int]$MinDepth = 4,
    [int]$MaxDepth = 9,
    [int]$Hash = 64,
    [int]$Threads = 1,
    [int]$PositionLimit = 0,
    [string]$OutFile = "",
    [int]$TimeoutMs = 1800000
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "harness_common.ps1")

if ($MinDepth -lt 1 -or $MaxDepth -lt $MinDepth) { throw "Require 1 <= MinDepth <= MaxDepth." }
if ($Hash -lt 1 -or $Threads -lt 1 -or $TimeoutMs -lt 1) { throw "Hash, Threads and TimeoutMs must be positive." }
foreach ($required in @($Engine, $Positions)) {
    if (-not (Test-Path -LiteralPath $required -PathType Leaf)) { throw "Missing: $required" }
}
$Engine = (Resolve-Path -LiteralPath $Engine).Path
$Positions = (Resolve-Path -LiteralPath $Positions).Path
foreach ($option in @("Hash", "Threads")) {
    if (-not (Test-EngineSupportsOption -Path $Engine -Name $option)) {
        throw "Engine does not advertise required UCI option '$option'."
    }
}

$fens = @(Get-Content -LiteralPath $Positions | ForEach-Object {
    $fen = ($_ -split '\s+;\s+', 2)[0].Trim()
    if ($fen -and -not $fen.StartsWith('#')) { $fen }
})
if ($PositionLimit -gt 0 -and $fens.Count -gt $PositionLimit) {
    $fens = @($fens | Select-Object -First $PositionLimit)
}
if ($fens.Count -eq 0) { throw "No positions found in $Positions." }

$script:engineProcess = $null
function Start-ProfileEngine {
    $psi = [System.Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = $Engine
    $psi.WorkingDirectory = Split-Path -Parent $Engine
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.UseShellExecute = $false
    $script:engineProcess = [System.Diagnostics.Process]::Start($psi)
}
function Stop-ProfileEngine {
    if ($null -eq $script:engineProcess) { return }
    if (-not $script:engineProcess.HasExited) {
        try { $script:engineProcess.StandardInput.WriteLine("quit") } catch {}
        $script:engineProcess.WaitForExit(5000) | Out-Null
    }
    if (-not $script:engineProcess.HasExited) { $script:engineProcess.Kill($true) }
    $script:engineProcess.Dispose()
    $script:engineProcess = $null
}
function Send-ProfileCommand([string]$Text) {
    $script:engineProcess.StandardInput.WriteLine($Text)
    $script:engineProcess.StandardInput.Flush()
}
function Wait-ProfileLine([string]$Pattern, [int]$LimitMs) {
    $deadline = [DateTime]::UtcNow.AddMilliseconds($LimitMs)
    $seen = [System.Collections.Generic.List[string]]::new()
    while ([DateTime]::UtcNow -lt $deadline) {
        $remaining = [Math]::Max(1, [int]($deadline - [DateTime]::UtcNow).TotalMilliseconds)
        $read = $script:engineProcess.StandardOutput.ReadLineAsync()
        if (-not $read.Wait($remaining)) { break }
        $line = $read.Result
        if ($null -eq $line) { throw "Engine closed stdout before /$Pattern/." }
        $seen.Add($line)
        if ($line -match $Pattern) { return $seen.ToArray() }
    }
    throw "Timed out after ${LimitMs} ms waiting for /$Pattern/."
}

$rows = @()
$positionRows = @(
    for ($positionIndex = 0; $positionIndex -lt $fens.Count; $positionIndex++) {
        [ordered]@{
            index = $positionIndex + 1
            fen   = $fens[$positionIndex]
            nodes = [ordered]@{}
        }
    }
)
$previous = [int64]0
$previousIteration = [int64]0
try {
    Write-Host "Engine:    $Engine"
    Write-Host "Positions: $($fens.Count) from $(Split-Path -Leaf $Positions)"
    Write-Host "Hash:      $Hash MiB   Threads: $Threads"
    $header = "{0,5} {1,16} {2,10} {3,12}" -f "depth", "nodes", "ratio", "time_ms"
    Write-Host "`n$header"
    Write-Host ("-" * $header.Length)

    foreach ($depth in $MinDepth..$MaxDepth) {
        Start-ProfileEngine
        Send-ProfileCommand "uci"; [void](Wait-ProfileLine '^uciok\s*$' 30000)
        Send-ProfileCommand "setoption name Hash value $Hash"
        Send-ProfileCommand "setoption name Threads value $Threads"
        Send-ProfileCommand "isready"; [void](Wait-ProfileLine '^readyok\s*$' 30000)
        $total = [int64]0
        $started = [DateTime]::UtcNow
        for ($positionIndex = 0; $positionIndex -lt $fens.Count; $positionIndex++) {
            $fen = $fens[$positionIndex]
            Send-ProfileCommand "ucinewgame"
            Send-ProfileCommand "isready"; [void](Wait-ProfileLine '^readyok\s*$' 60000)
            Send-ProfileCommand "position fen $fen"
            Send-ProfileCommand "go depth $depth"
            $lines = Wait-ProfileLine '^bestmove\b' $TimeoutMs
            $nodes = [int64]0
            foreach ($line in $lines) {
                if ($line -match '^info .*\bnodes\s+(\d+)') { $nodes = [int64]$Matches[1] }
            }
            if ($nodes -le 0) { throw "No node count for depth $depth on: $fen" }
            $positionRows[$positionIndex].nodes["$depth"] = $nodes
            $total += $nodes
        }
        $elapsed = [int64]([DateTime]::UtcNow - $started).TotalMilliseconds
        Stop-ProfileEngine
        $ratio = if ($previous -gt 0) { [Math]::Round($total / $previous, 3) } else { [double]::NaN }
        $iterationNodes = if ($previous -gt 0) { $total - $previous } else { $null }
        $iterationGrowth = if ($null -ne $iterationNodes -and $previousIteration -gt 0 -and $iterationNodes -gt 0) {
            [Math]::Round($iterationNodes / $previousIteration, 3)
        } else { $null }
        $rows += [pscustomobject]@{
            depth            = $depth
            nodes            = $total
            ratio            = $ratio
            iteration_nodes  = $iterationNodes
            iteration_growth = $iterationGrowth
            time_ms          = $elapsed
        }
        Write-Host ("{0,5} {1,16:N0} {2,10} {3,12:N0}" -f $depth, $total,
            $(if ([double]::IsNaN($ratio)) { "-" } else { $ratio }), $elapsed)
        if ($null -ne $iterationNodes -and $iterationNodes -gt 0) { $previousIteration = $iterationNodes }
        $previous = $total
    }

    $ratios = @($rows | Where-Object { -not [double]::IsNaN($_.ratio) })
    $geometric = $null
    if ($ratios.Count -gt 0) {
        $sumLogs = ($ratios | ForEach-Object { [Math]::Log($_.ratio) } | Measure-Object -Sum).Sum
        $geometric = [Math]::Round([Math]::Exp($sumLogs / $ratios.Count), 3)
        Write-Host "`nGeometric mean consecutive-depth ratio: $geometric"
    }

    $positionProfiles = @()
    $positionSpanRatios = @()
    $span = $MaxDepth - $MinDepth
    foreach ($position in $positionRows) {
        $depthRows = @()
        $priorNodes = [int64]0
        $priorIteration = [int64]0
        foreach ($depth in $MinDepth..$MaxDepth) {
            $nodes = [int64]$position.nodes["$depth"]
            $ratio = if ($priorNodes -gt 0) { [Math]::Round($nodes / $priorNodes, 6) } else { $null }
            $iterationNodes = if ($priorNodes -gt 0) { $nodes - $priorNodes } else { $null }
            $iterationGrowth = if ($null -ne $iterationNodes -and $priorIteration -gt 0 -and $iterationNodes -gt 0) {
                [Math]::Round($iterationNodes / $priorIteration, 6)
            } else { $null }
            $depthRows += [pscustomobject]@{
                depth            = $depth
                nodes            = $nodes
                ratio            = $ratio
                iteration_nodes  = $iterationNodes
                iteration_growth = $iterationGrowth
            }
            if ($null -ne $iterationNodes -and $iterationNodes -gt 0) { $priorIteration = $iterationNodes }
            $priorNodes = $nodes
        }
        $firstNodes = [int64]$position.nodes["$MinDepth"]
        $lastNodes = [int64]$position.nodes["$MaxDepth"]
        $spanRatio = if ($span -gt 0 -and $firstNodes -gt 0 -and $lastNodes -gt 0) {
            [Math]::Round([Math]::Pow($lastNodes / $firstNodes, 1.0 / $span), 6)
        } else { $null }
        if ($null -ne $spanRatio) { $positionSpanRatios += $spanRatio }
        $positionProfiles += [pscustomobject]@{
            index      = $position.index
            fen        = $position.fen
            span_ratio = $spanRatio
            rows       = $depthRows
        }
    }

    $positionMedian = $null
    $positionMin = $null
    $positionMax = $null
    if ($positionSpanRatios.Count -gt 0) {
        $sorted = @($positionSpanRatios | Sort-Object)
        $middle = [int]($sorted.Count / 2)
        $positionMedian = if (($sorted.Count % 2) -eq 1) {
            $sorted[$middle]
        } else {
            [Math]::Round(($sorted[$middle - 1] + $sorted[$middle]) / 2.0, 6)
        }
        $positionMin = $sorted[0]
        $positionMax = $sorted[-1]
        Write-Host ("Per-position span ratio: median {0:N3}  min {1:N3}  max {2:N3}" -f `
            $positionMedian, $positionMin, $positionMax)
    }
    if ($OutFile) {
        $report = [ordered]@{
            schema = "rarog-branching-profile-v2"
            created_utc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
            engine = $Engine
            engine_sha256 = Get-HarnessSha256 $Engine
            positions = $Positions
            positions_sha256 = Get-HarnessSha256 $Positions
            position_count = $fens.Count
            hash_mb = $Hash
            threads = $Threads
            min_depth = $MinDepth
            max_depth = $MaxDepth
            geometric_mean_ratio = $geometric
            per_position_span = [ordered]@{
                median = $positionMedian
                min    = $positionMin
                max    = $positionMax
            }
            rows = $rows
            positions_detail = $positionProfiles
        }
        $parent = Split-Path -Parent ([IO.Path]::GetFullPath($OutFile))
        if ($parent -and -not (Test-Path -LiteralPath $parent)) {
            New-Item -ItemType Directory -Force -Path $parent | Out-Null
        }
        Write-JsonAtomic -Path ([IO.Path]::GetFullPath($OutFile)) -Value $report
        Write-Host "Report -> $OutFile"
    }
} finally {
    Stop-ProfileEngine
}
