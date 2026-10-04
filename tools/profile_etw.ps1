<#
.SYNOPSIS
    ETW/xperf sampling profile of a Rarog bench run. REQUIRES AN ELEVATED SHELL.

.DESCRIPTION
    Kernel CPU sampling with stack walking needs administrator rights, which the
    model's shell does not have — so this is the one profiling step a human runs.
    Everything else (building the symbol-bearing binary, reading the reports) is
    already done or happens afterwards.

    The script is self-contained and restores machine state in a `finally`
    block: it always stops the kernel session and resets the sampling interval,
    even if the workload fails.

    Binary profiled: tools\profile\rarog.exe — a release build with `debug=2`
    (PDB alongside). Debug info lives in the PDB and does NOT change codegen;
    verified bench-identical at 5,480,624.

.PARAMETER Repeats
    bench repeats. Default 5 (~10 s), which at 8 kHz gives ~80k samples.

.PARAMETER SampleIntervalNs100
    Sampling interval in 100 ns units. 1221 ≈ 8 kHz (default here);
    10000 = 1 ms = 1 kHz is the Windows default.

.EXAMPLE
    # In an ELEVATED PowerShell 7 window:
    pwsh -File D:\code\rarog\tools\profile_etw.ps1
#>
param(
    [int]$Repeats = 5,
    [int]$SampleIntervalNs100 = 1221
)

$ErrorActionPreference = 'Stop'

# --- preflight ------------------------------------------------------------
$id = [Security.Principal.WindowsIdentity]::GetCurrent()
if (-not (New-Object Security.Principal.WindowsPrincipal($id)).IsInRole(
        [Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "Not elevated. Re-run this script from an Administrator PowerShell 7 window."
}

$xperf = "C:\Program Files (x86)\Windows Kits\10\Windows Performance Toolkit\xperf.exe"
if (-not (Test-Path $xperf)) { throw "xperf not found at: $xperf" }

$profDir = "D:\code\rarog\tools\profile"
$exe     = Join-Path $profDir "rarog.exe"
$pdb     = Join-Path $profDir "rarog.pdb"
$manifest = Join-Path $profDir "rarog.json"
foreach ($f in @($exe, $pdb, $manifest)) { if (-not (Test-Path $f)) { throw "missing: $f" } }

# STALENESS GUARD. The staged binary is copied here by hand, so without this
# check the script silently profiles whatever was last copied. That happened
# once: a trace taken to measure a change was actually of the binary before
# it and was meaningless.
#
# The test is on BUILD INPUTS, not on HEAD. Comparing HEAD rejects the binary
# after any docs- or tooling-only commit, which is a false alarm that trains
# you to ignore the guard — exactly what we do not want from a safety check.
$repo = "D:\code\rarog"
$buildPaths = @("src", "Cargo.toml", "Cargo.lock", "build.rs", "rust-toolchain.toml")
$ids = foreach ($bp in $buildPaths) { (& git -C $repo rev-parse "HEAD:$bp").Trim() }
$sourceId = ($ids -join "-")

$meta = Get-Content $manifest -Raw | ConvertFrom-Json
if ($meta.source_id -ne $sourceId) {
    throw ("Staged profiling binary is STALE — build inputs changed since it was built.`n" +
           "Rebuild it:`n" +
           "  CARGO_PROFILE_RELEASE_DEBUG=2 cargo build --release`n" +
           "  cp target/release/rarog.exe tools/profile/rarog.exe`n" +
           "  cp target/release/rarog.pdb tools/profile/rarog.pdb`n" +
           "  then refresh tools/profile/rarog.json (see the model, or rerun its staging step)")
}

# Uncommitted edits to build inputs would also make the binary lie.
$dirtyBuild = (& git -C $repo status --porcelain -- $buildPaths) -join ""
if ($dirtyBuild.Trim()) {
    throw "Build inputs have UNCOMMITTED changes - the staged binary does not match the source. Rebuild and restage it."
}
Write-Host ("binary   : matches current build inputs (built at " + $meta.git_sha.Substring(0,8) + "), bench " + $meta.bench_nodes)

$etl        = Join-Path $profDir "rarog_cpu.etl"
$flatReport = Join-Path $profDir "profile_flat.txt"
$stackReport= Join-Path $profDir "profile_stacks.txt"

# Local PDB only. Deliberately NOT pointing at the Microsoft symbol server:
# we care about rarog.exe frames, and a symbol-server fetch can take minutes
# (or hang offline). Unresolved ntoskrnl/ntdll frames are fine.
$env:_NT_SYMBOL_PATH = $profDir

Write-Host "=== Rarog ETW profile ===" -ForegroundColor Cyan
Write-Host "binary   : $exe"
Write-Host "workload : bench 13 $Repeats"
Write-Host "interval : $SampleIntervalNs100 (100ns units) = $([math]::Round(10000000.0/$SampleIntervalNs100)) Hz"
Write-Host ""

try {
    # Clear any stale kernel session from an interrupted earlier run.
    & $xperf -stop 2>&1 | Out-Null

    & $xperf -setprofint $SampleIntervalNs100
    if ($LASTEXITCODE -ne 0) { throw "xperf -setprofint failed ($LASTEXITCODE)" }

    & $xperf -on PROC_THREAD+LOADER+PROFILE -stackwalk profile `
             -buffersize 1024 -minbuffers 256 -maxbuffers 1024
    if ($LASTEXITCODE -ne 0) { throw "xperf -on failed ($LASTEXITCODE)" }

    Write-Host "tracing... running workload (leave the machine otherwise IDLE)" -ForegroundColor Yellow
    $out = "bench 13 $Repeats" | & $exe 2>&1
    $out | Select-String "Nodes searched|Nodes/second" | ForEach-Object { Write-Host "  $_" }

    & $xperf -stop -d $etl
    if ($LASTEXITCODE -ne 0) { throw "xperf -stop failed ($LASTEXITCODE)" }
    Write-Host "trace written: $etl"
}
finally {
    # Always leave the machine as we found it.
    & $xperf -stop 2>&1 | Out-Null
    & $xperf -setprofint 10000 2>&1 | Out-Null
}

# --- reports --------------------------------------------------------------
# `-a profile` produces a per-CPU UTILISATION TIMELINE, not a symbol profile —
# that mistake cost a whole capture once. The symbol-level report is
# `-a stack -butterfly`, restricted to our process. Errors are NOT swallowed.
Write-Host ""
Write-Host "resolving symbols (first run may take a minute)..." -ForegroundColor Yellow

$butterfly = Join-Path $profDir "butterfly.txt"
& $xperf -i $etl -o $butterfly -symbols -a stack -butterfly 50 -process "rarog.exe"
if ($LASTEXITCODE -ne 0) { throw "xperf stack report failed ($LASTEXITCODE)" }
if (-not (Test-Path $butterfly)) { throw "stack report produced no file" }
$size = (Get-Item $butterfly).Length
if ($size -lt 10000) { throw "stack report is suspiciously small ($size bytes) - symbols likely failed" }

Write-Host ""
Write-Host ("stack/butterfly report: {0} ({1:N0} bytes)" -f $butterfly, $size) -ForegroundColor Green
Write-Host "Total samples in rarog.exe:" -ForegroundColor Cyan
$html = Get-Content $butterfly -Raw
if ($html -match "<td>rarog\.exe</td><td>\d+</td><td>(\d+)</td>") {
    Write-Host ("  {0:N0}" -f [int]$Matches[1])
}
Write-Host ""
Write-Host "Done. Tell the model the report is ready; it reads it from:" -ForegroundColor Green
Write-Host "  $butterfly"
