<#
.SYNOPSIS
    Prove tools/sprt.ps1's per-engine search limits refuse every combination
    that makes no sense, and let the sensible ones through.

.DESCRIPTION
    Each negative case breaks one rule and requires the refusal that names it.
    The positive controls give valid limits and a placeholder engine path, so a
    case that passes the limit checks stops at the engine check ("Not found")
    instead: no game is played and fastchess is never started. A script that
    refused everything could not pass the positive controls.

        pwsh -NoProfile -File tools/diag/test_sprt_limits.ps1

    Exit status 0 means every case behaved; 1 names the cases that did not.
#>
$ErrorActionPreference = "Stop"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$wrapper = Join-Path $repo "tools\sprt.ps1"
$missing = Join-Path ([System.IO.Path]::GetTempPath()) ("rarog-no-engine-" + [guid]::NewGuid() + ".exe")
$base = @{ EngineA = $missing; EngineB = $missing; Mode = "fixed"; Games = 2; Concurrency = 1 }

$cases = @(
    @{ Name = "two kinds: nodes against move time"; Args = @{ NodesA = 1000; MoveTimeB = 0.1 }; Expect = "two kinds" }
    @{ Name = "two kinds: own clock against shared nodes"; Args = @{ TCA = "1+0.01"; Nodes = 5000 }; Expect = "two kinds" }
    @{ Name = "two kinds: own nodes against the default clock"; Args = @{ NodesA = 1000 }; Expect = "two kinds" }
    @{ Name = "two limits for one engine"; Args = @{ NodesA = 1000; MoveTimeA = 0.1 }; Expect = "more than one limit" }
    @{ Name = "a shared limit neither engine reads"; Args = @{ NodesA = 1000; NodesB = 2000; Nodes = 5000 }; Expect = "read by neither" }
    @{ Name = "a shared clock neither engine reads"; Args = @{ TCA = "1+0.01"; TCB = "2+0.02"; TC = "3+0.03" }; Expect = "read by neither" }
    @{ Name = "a per-engine limit in a calibration"; Args = @{ NodesA = 1000; NodesB = 1000; Mode = "calibrate" }; Expect = "null pair" }
    @{ Name = "a negative node budget"; Args = @{ NodesA = -5; NodesB = 1000 }; Expect = "must be positive" }
    @{ Name = "an explicit zero move time"; Args = @{ MoveTimeA = 0; MoveTimeB = 0.1 }; Expect = "must be positive" }
    @{ Name = "a malformed clock"; Args = @{ TCA = "3s"; TCB = "3+0.03" }; Expect = "is not <seconds>+<increment seconds>" }
    @{ Name = "shared nodes and move time"; Args = @{ Nodes = 1000; MoveTime = 0.1 }; Expect = "mutually exclusive" }
    @{ Name = "control: per-engine nodes"; Args = @{ NodesA = 1000; NodesB = 4000 }; Expect = "Not found" }
    @{ Name = "control: per-engine move times"; Args = @{ MoveTimeA = 0.05; MoveTimeB = 0.2 }; Expect = "Not found" }
    @{ Name = "control: one own budget, the other shared"; Args = @{ NodesA = 1000; Nodes = 4000 }; Expect = "Not found" }
    @{ Name = "control: per-engine clocks"; Args = @{ TCA = "1+0.01"; TCB = "3+0.03" }; Expect = "Not found" }
    @{ Name = "control: shared nodes"; Args = @{ Nodes = 1000 }; Expect = "Not found" }
)

$failed = @()
foreach ($case in $cases) {
    $arguments = $base.Clone()
    foreach ($key in $case.Args.Keys) { $arguments[$key] = $case.Args[$key] }
    if ($arguments.Mode -eq "calibrate") { $arguments.Remove("Games") }
    $message = $null
    try {
        & $wrapper @arguments *> $null
        $message = "(no refusal)"
    } catch {
        $message = $_.Exception.Message
    }
    if ($message -like "*$($case.Expect)*") {
        Write-Host "ok    $($case.Name)"
    } else {
        Write-Host "FAIL  $($case.Name): expected '$($case.Expect)', got: $message" -ForegroundColor Red
        $failed += $case.Name
    }
}
Write-Host "$($cases.Count - $failed.Count)/$($cases.Count) cases behaved"
if ($failed.Count -gt 0) { exit 1 }
exit 0
