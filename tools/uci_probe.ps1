# Minimal UCI driver for scaling measurements.
# Traps honoured (analysis/archive/smp_analysis.md): live process (never pipe `go ... quit`),
# `ucinewgame` + `isready` before every `go` so no reading is TT-warmed by the
# previous one, and read until `bestmove`.

function Start-Engine([string]$exe) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $exe
    $psi.WorkingDirectory = (Split-Path $exe -Parent)
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    return [System.Diagnostics.Process]::Start($psi)
}

function Send-Line($p, [string]$line) {
    $p.StandardInput.WriteLine($line)
    $p.StandardInput.Flush()
}

function Read-Until($p, [string]$pattern, [int]$timeoutSec = 60) {
    $lines = New-Object System.Collections.ArrayList
    $deadline = (Get-Date).AddSeconds($timeoutSec)
    while ($true) {
        if ((Get-Date) -gt $deadline) { break }
        if ($p.HasExited -and $p.StandardOutput.EndOfStream) { break }
        $task = $p.StandardOutput.ReadLineAsync()
        $remaining = [int]([Math]::Max(1, ($deadline - (Get-Date)).TotalMilliseconds))
        if (-not $task.Wait($remaining)) { break }
        $line = $task.Result
        if ($null -eq $line) { break }
        [void]$lines.Add($line)
        if ($line -match $pattern) { break }
    }
    return $lines
}

function Get-UciOptions([string]$exe) {
    $p = Start-Engine $exe
    Send-Line $p "uci"
    $lines = Read-Until $p "^uciok" 20
    Send-Line $p "quit"
    Start-Sleep -Milliseconds 200
    if (-not $p.HasExited) { $p.Kill() }
    return $lines | Where-Object { $_ -match '^option name' }
}

# One cold-start search. Returns the NPS the engine itself reports on its last
# info line, falling back to nodes/time when an engine omits `nps`.
function Measure-Search([string]$exe, [string]$fen, [int]$threads, [int]$hashMb,
    [int]$movetimeMs, [string]$threadOpt, [string]$hashOpt) {
    $p = Start-Engine $exe
    Send-Line $p "uci"
    [void](Read-Until $p "^uciok" 20)
    if ($hashOpt) { Send-Line $p "setoption name $hashOpt value $hashMb" }
    if ($threadOpt) { Send-Line $p "setoption name $threadOpt value $threads" }
    Send-Line $p "ucinewgame"
    Send-Line $p "isready"
    [void](Read-Until $p "^readyok" 60)
    if ($fen -eq "startpos") { Send-Line $p "position startpos" }
    else { Send-Line $p "position fen $fen" }
    Send-Line $p "go movetime $movetimeMs"
    $out = Read-Until $p "^bestmove" ([int]($movetimeMs / 1000) + 45)
    Send-Line $p "quit"
    Start-Sleep -Milliseconds 150
    if (-not $p.HasExited) { $p.Kill() }

    $info = $out | Where-Object { $_ -match '^info .*\bnodes\b' } | Select-Object -Last 1
    if (-not $info) { return $null }
    $nps = $null
    if ($info -match '\bnps\s+(\d+)') { $nps = [double]$Matches[1] }
    elseif ($info -match '\bnodes\s+(\d+)' ) {
        $nodes = [double]$Matches[1]
        if ($info -match '\btime\s+(\d+)') { $nps = $nodes * 1000.0 / [double]$Matches[1] }
    }
    $depth = if ($info -match '\bdepth\s+(\d+)') { [int]$Matches[1] } else { 0 }
    return [pscustomobject]@{ Nps = $nps; Depth = $depth }
}
