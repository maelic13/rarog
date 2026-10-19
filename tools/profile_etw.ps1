<#
.SYNOPSIS
    ETW/xperf sampling profiles of Rarog: a bench run and a game-play sample.
    REQUIRES AN ELEVATED SHELL (except -DryRun).

.DESCRIPTION
    Kernel CPU sampling with stack walking needs administrator rights, which the
    model's shell does not have — so this is the one profiling step a human runs.
    Everything else (building the symbol-bearing binary, reading the reports) is
    already done or happens afterwards.

    Two captures, each with its own trace and report:
      bench  — `bench 13 <Repeats>`: 40 fixed positions, each from an empty hash.
      games  — tools/profile_games.py: every ply of <Games> games from a PGN at
               `go nodes <GameNodes>`, the hash carried through each game. This
               is the shape a match plays; bench is not.
    -Workload picks one; the default runs both.

    The script restores machine state in a `finally` block: it always stops the
    kernel session and resets the sampling interval, even if a workload fails.

    Binary profiled: tools\profile\rarog.exe — a release build with `debug=2`
    (PDB alongside). Debug info lives in the PDB and does NOT change codegen;
    verified bench-identical at 5,480,624.

.PARAMETER Repeats
    bench repeats. Default 5 (~10 s), which at 8 kHz gives ~80k samples.

.PARAMETER Games
    Games replayed in the game capture. Default 6 (~40 s).

.PARAMETER GameNodes
    Nodes per search in the game capture. Default 150,000, about one move's
    search at 3+0.03 on this host.

.PARAMETER GamePgn
    Source of the game capture's games.

.PARAMETER Workload
    all (default), bench or games.

.PARAMETER SampleIntervalNs100
    Sampling interval in 100 ns units. 1221 ≈ 8 kHz (default here);
    10000 = 1 ms = 1 kHz is the Windows default.

.PARAMETER DryRun
    Run the preflight and every workload without tracing: no elevation, no
    xperf session, no report. Proves the command works before a real capture.

.EXAMPLE
    # In an ELEVATED PowerShell 7 window:
    pwsh -File D:\code\rarog\tools\profile_etw.ps1
#>
param(
    [int]$Repeats = 5,
    [int]$Games = 6,
    [int]$GameNodes = 150000,
    [string]$GamePgn = "D:\code\rarog\tools\results\rar-m72\games.pgn",
    [ValidateSet("all", "bench", "games")][string]$Workload = "all",
    [int]$SampleIntervalNs100 = 1221,
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'

# --- preflight ------------------------------------------------------------
if (-not $DryRun) {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    if (-not (New-Object Security.Principal.WindowsPrincipal($id)).IsInRole(
            [Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw "Not elevated. Re-run this script from an Administrator PowerShell 7 window."
    }
}

$xperf = "C:\Program Files (x86)\Windows Kits\10\Windows Performance Toolkit\xperf.exe"
if (-not (Test-Path $xperf)) { throw "xperf not found at: $xperf" }

$repo     = "D:\code\rarog"
$profDir  = Join-Path $repo "tools\profile"
$exe      = Join-Path $profDir "rarog.exe"
$pdb      = Join-Path $profDir "rarog.pdb"
$manifest = Join-Path $profDir "rarog.json"
foreach ($f in @($exe, $pdb, $manifest)) { if (-not (Test-Path $f)) { throw "missing: $f" } }

$runGames = $Workload -ne "bench"
if ($runGames) {
    $python = (Get-Command python -ErrorAction SilentlyContinue).Source
    if (-not $python) { throw "python not found on PATH (the game capture needs it, with python-chess)" }
    $driver = Join-Path $repo "tools\profile_games.py"
    if (-not (Test-Path $GamePgn)) { throw "game PGN not found: $GamePgn" }
}

# STALENESS GUARD. The staged binary is copied here by hand, so without this
# check the script silently profiles whatever was last copied. That happened
# once: a trace taken to measure a change was actually of the binary before
# it and was meaningless.
#
# The test is on BUILD INPUTS, not on HEAD. Comparing HEAD rejects the binary
# after any docs- or tooling-only commit, which is a false alarm that trains
# you to ignore the guard — exactly what we do not want from a safety check.
$buildPaths = @("src", "Cargo.toml", "Cargo.lock", "build.rs", "rust-toolchain.toml")
$ids = foreach ($bp in $buildPaths) { (& git -C $repo rev-parse "HEAD:$bp").Trim() }
$sourceId = ($ids -join "-")

$meta = Get-Content $manifest -Raw | ConvertFrom-Json
if ($meta.source_id -ne $sourceId) {
    throw ("Staged profiling binary is STALE — build inputs changed since it was built.`n" +
           "Rebuild it with symbols (CARGO_PROFILE_RELEASE_DEBUG=2 and the build in`n" +
           "tools/profile/rarog.json's `"build`" field), copy rarog.exe and rarog.pdb`n" +
           "into tools/profile, and refresh rarog.json's source_id, git_sha and bench_nodes.")
}

# Uncommitted edits to build inputs would also make the binary lie.
$dirtyBuild = (& git -C $repo status --porcelain -- $buildPaths) -join ""
if ($dirtyBuild.Trim()) {
    throw "Build inputs have UNCOMMITTED changes - the staged binary does not match the source. Rebuild and restage it."
}

# The staged binary must also be the one rarog.json describes.
$hash = (Get-FileHash -Algorithm SHA256 $exe).Hash
if ($meta.sha256 -and $meta.sha256 -ne $hash) {
    throw "tools/profile/rarog.exe is not the binary rarog.json names (sha256 $hash, expected $($meta.sha256))."
}
Write-Host ("binary   : matches current build inputs (built at " + $meta.git_sha.Substring(0,8) + "), bench " + $meta.bench_nodes)

# Local PDB only. Deliberately NOT pointing at the Microsoft symbol server:
# we care about rarog.exe frames, and a symbol-server fetch can take minutes
# (or hang offline). Unresolved ntoskrnl/ntdll frames are fine.
$env:_NT_SYMBOL_PATH = $profDir

# --- workloads ------------------------------------------------------------
# Each throws on failure, so a broken workload never yields a trace.
function Invoke-Bench {
    $out = "bench 13 $Repeats" | & $exe 2>&1
    $lines = $out | Select-String "Nodes searched|Nodes/second"
    if (-not $lines) { throw "bench printed no summary" }
    $lines | ForEach-Object { Write-Host "  $_" }
    $nodes = ($out | Select-String "Nodes searched\s*:?\s*(\d+)" | Select-Object -First 1).Matches.Groups[1].Value
    if ($nodes -and $meta.bench_nodes -and [long]$nodes -ne [long]$meta.bench_nodes) {
        throw "bench searched $nodes nodes, not the staged binary's $($meta.bench_nodes)"
    }
}

function Invoke-Games {
    & $python $driver --engine $exe --pgn $GamePgn --games $Games --nodes $GameNodes |
        ForEach-Object { Write-Host "  $_" }
    if ($LASTEXITCODE -ne 0) { throw "game workload failed ($LASTEXITCODE)" }
}

$captures = @()
if ($Workload -ne "games") { $captures += @{ Name = "bench"; Run = ${function:Invoke-Bench}; What = "bench 13 $Repeats" } }
if ($runGames) { $captures += @{ Name = "games"; Run = ${function:Invoke-Games}; What = "$Games games at go nodes $GameNodes from $GamePgn" } }

Write-Host "=== Rarog ETW profile$(if ($DryRun) { ' (DRY RUN: no tracing)' }) ===" -ForegroundColor Cyan
Write-Host "binary   : $exe"
Write-Host "interval : $SampleIntervalNs100 (100ns units) = $([math]::Round(10000000.0/$SampleIntervalNs100)) Hz"
Write-Host ""

$reports = @()
foreach ($c in $captures) {
    $etl       = Join-Path $profDir "rarog_cpu_$($c.Name).etl"
    $butterfly = Join-Path $profDir "butterfly_$($c.Name).txt"
    Write-Host "--- capture '$($c.Name)': $($c.What)" -ForegroundColor Cyan

    if ($DryRun) {
        & $c.Run
        Write-Host "  workload OK (no trace taken)"
        continue
    }

    try {
        # Clear any stale kernel session from an interrupted earlier run.
        & $xperf -stop 2>&1 | Out-Null

        & $xperf -setprofint $SampleIntervalNs100
        if ($LASTEXITCODE -ne 0) { throw "xperf -setprofint failed ($LASTEXITCODE)" }

        & $xperf -on PROC_THREAD+LOADER+PROFILE -stackwalk profile `
                 -buffersize 1024 -minbuffers 256 -maxbuffers 1024
        if ($LASTEXITCODE -ne 0) { throw "xperf -on failed ($LASTEXITCODE)" }

        Write-Host "tracing... running workload (leave the machine otherwise IDLE)" -ForegroundColor Yellow
        & $c.Run

        & $xperf -stop -d $etl
        if ($LASTEXITCODE -ne 0) { throw "xperf -stop failed ($LASTEXITCODE)" }
        Write-Host "trace written: $etl"
    }
    finally {
        # Always leave the machine as we found it.
        & $xperf -stop 2>&1 | Out-Null
        & $xperf -setprofint 10000 2>&1 | Out-Null
    }

    # `-a profile` produces a per-CPU UTILISATION TIMELINE, not a symbol profile —
    # that mistake cost a whole capture once. The symbol-level report is
    # `-a stack -butterfly`, restricted to our process. Errors are NOT swallowed.
    Write-Host "resolving symbols (first run may take a minute)..." -ForegroundColor Yellow
    & $xperf -i $etl -o $butterfly -symbols -a stack -butterfly 50 -process "rarog.exe"
    if ($LASTEXITCODE -ne 0) { throw "xperf stack report failed ($LASTEXITCODE)" }
    if (-not (Test-Path $butterfly)) { throw "stack report produced no file" }
    $size = (Get-Item $butterfly).Length
    if ($size -lt 10000) { throw "stack report is suspiciously small ($size bytes) - symbols likely failed" }

    $samples = ""
    $html = Get-Content $butterfly -Raw
    if ($html -match "<td>rarog\.exe</td><td>\d+</td><td>(\d+)</td>") { $samples = "{0:N0}" -f [int]$Matches[1] }
    Write-Host ("report: {0} ({1:N0} bytes, {2} samples in rarog.exe)" -f $butterfly, $size, $samples) -ForegroundColor Green
    $reports += $butterfly
    Write-Host ""
}

if ($DryRun) {
    Write-Host "Dry run passed: every workload ran; no trace was taken." -ForegroundColor Green
} else {
    Write-Host "Done. Tell the model the reports are ready; it reads them from:" -ForegroundColor Green
    $reports | ForEach-Object { Write-Host "  $_" }
}
