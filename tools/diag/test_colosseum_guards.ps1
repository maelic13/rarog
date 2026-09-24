<#
.SYNOPSIS
    Prove every guard in tools/colosseum.ps1 fires on a deliberately wrong input.

.DESCRIPTION
    A guard nobody has watched fail is a comment. Each case below breaks exactly
    one input and requires the wrapper to refuse with the message that names it;
    the positive controls require the same commands to pass when nothing is
    broken, so a wrapper that refused everything could not score a pass here
    either.

    Every case is a dry run: no game is played and no run directory is created.
    Sidecars are mutated on COPIES in a scratch directory, never in
    tools/test_engines.

        pwsh -NoProfile -File tools/diag/test_colosseum_guards.ps1

    Exit status 0 means every case behaved; 1 names the cases that did not.
#>
param(
    [string]$EngineA = "tools/test_engines/rarog-b24b-core-pext-pgo.exe",
    [string]$EngineB = "tools/test_engines/rarog-b23-theta3900-pext-pgo.exe",
    [string]$TuneEngine = "tools/test_engines/rarog-b23core-tune.exe",
    [string]$OffArmTuneEngine = "tools/test_engines/rarog-b23negoff-tune.exe",
    [switch]$KeepScratch
)

$ErrorActionPreference = "Stop"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$wrapper = Join-Path $repo "tools\colosseum.ps1"
$pin = Join-Path $repo "tools\colosseum\colosseum.pin.json"

foreach ($required in @($EngineA, $EngineB, $TuneEngine, $OffArmTuneEngine)) {
    if (-not (Test-Path -LiteralPath (Join-Path $repo $required))) {
        throw "This suite needs $required, which is not staged on this host."
    }
}

$scratch = Join-Path ([System.IO.Path]::GetTempPath()) ("rarog-guards-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $scratch | Out-Null

function Merge-Arguments {
    # PowerShell's hashtable `+` throws on a duplicate key, which is exactly what
    # an override is, so the cases build their argument sets through here.
    param([hashtable]$Base, [hashtable]$Override = @{}, [string[]]$Remove = @())
    $merged = $Base.Clone()
    foreach ($key in $Override.Keys) { $merged[$key] = $Override[$key] }
    foreach ($key in $Remove) { $merged.Remove($key) }
    $merged
}

function Copy-Arm {
    # A binary plus its sidecar, in the scratch directory, so a case can corrupt
    # the sidecar without touching the staged asset.
    param([string]$Source, [string]$Name)
    $sourcePath = Join-Path $repo $Source
    $target = Join-Path $scratch "$Name.exe"
    Copy-Item -LiteralPath $sourcePath -Destination $target -Force
    $manifest = Get-Content -LiteralPath ([IO.Path]::ChangeExtension($sourcePath, ".json")) -Raw | ConvertFrom-Json
    $manifest.engine = "$Name.exe"
    $manifest | ConvertTo-Json -Depth 8 |
        Set-Content -LiteralPath ([IO.Path]::ChangeExtension($target, ".json")) -Encoding utf8
    $target
}

function Set-Sidecar {
    param([string]$EnginePath, [hashtable]$Fields)
    $sidecar = [IO.Path]::ChangeExtension($EnginePath, ".json")
    $manifest = Get-Content -LiteralPath $sidecar -Raw | ConvertFrom-Json
    foreach ($key in $Fields.Keys) {
        $manifest | Add-Member -NotePropertyName $key -NotePropertyValue $Fields[$key] -Force
    }
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $sidecar -Encoding utf8
}

$results = [System.Collections.Generic.List[object]]::new()

function Invoke-Case {
    # `Expect` is a substring of the refusal that names the guard. An empty
    # Expect means the case must SUCCEED — a positive control.
    # `Check` replaces the wrapper call for a guard tested as a function.
    param([string]$Name, [string]$Expect, [hashtable]$Arguments, [scriptblock]$Check)

    $outcome = [pscustomobject]@{ Name = $Name; Expect = $Expect; Passed = $false; Detail = "" }
    try {
        if ($Check) { & $Check } else { & $wrapper @Arguments *>&1 | Out-Null }
        if ($Expect) {
            $outcome.Detail = "the wrapper ACCEPTED an input it should have refused"
        } else {
            $outcome.Passed = $true
            $outcome.Detail = "accepted, as the control requires"
        }
    } catch {
        $message = "$($_.Exception.Message)"
        if (-not $Expect) {
            $outcome.Detail = "the control was REFUSED: $message"
        } elseif ($message -like "*$Expect*") {
            $outcome.Passed = $true
            $outcome.Detail = "refused: $($message.Split([char]10)[0].Trim())"
        } else {
            $outcome.Detail = "refused for the WRONG reason: $message"
        }
    }
    $results.Add($outcome)
    $mark = if ($outcome.Passed) { "PASS" } else { "FAIL" }
    $colour = if ($outcome.Passed) { "Green" } else { "Red" }
    Write-Host ("  {0}  {1,-36} {2}" -f $mark, $Name, $outcome.Detail) -ForegroundColor $colour
}

try {
    $armA = Copy-Arm -Source $EngineA -Name "armA"
    $armB = Copy-Arm -Source $EngineB -Name "armB"
    $base = @{
        Mode = "sprt"; EngineA = $armA; EngineB = $armB
        NameA = "armA"; NameB = "armB"; MaxPairs = 100; Seed = 7
        Dir = (Join-Path $scratch "run"); DryRun = $true; AllowBusyHost = $true
    }
    $tune = @{
        Mode = "spsa"; Engine = (Copy-Arm -Source $TuneEngine -Name "tuneArm")
        ConfigGroup = "b23core"; Iterations = 5000; TotalGames = 150000; Seed = 7
        Dir = (Join-Path $scratch "tune"); DryRun = $true; AllowBusyHost = $true
    }

    Write-Host ""
    Write-Host "Guard cases (dry runs; no game is played):"

    Invoke-Case -Name "control: a clean gate resolves" -Expect "" -Arguments $base

    $stale = Copy-Arm -Source $EngineA -Name "stale"
    Set-Sidecar -EnginePath $stale -Fields @{ binary_sha256 = ("0" * 64) }
    Invoke-Case -Name "stale sidecar hash" -Expect "PROVENANCE MISMATCH" `
        -Arguments (Merge-Arguments $base @{ EngineA = $stale; NameA = "stale" })

    $unverified = Copy-Arm -Source $EngineA -Name "unverified"
    Set-Sidecar -EnginePath $unverified -Fields @{ verification = "none" }
    Invoke-Case -Name "sidecar without a bench" -Expect "not bench verification" `
        -Arguments (Merge-Arguments $base @{ EngineA = $unverified; NameA = "unverified" })

    $dirty = Copy-Arm -Source $EngineA -Name "dirty"
    Set-Sidecar -EnginePath $dirty -Fields @{ git_dirty = $true }
    Invoke-Case -Name "binary built from a dirty tree" -Expect "DIRTY TREE" `
        -Arguments (Merge-Arguments $base @{ EngineA = $dirty; NameA = "dirty" })
    Invoke-Case -Name "dirty tree, deliberately waived" -Expect "" `
        -Arguments (Merge-Arguments $base @{ EngineA = $dirty; NameA = "dirty"; AllowDirtyTree = $true })

    Invoke-Case -Name "-ExpectRevision names another commit" -Expect "WRONG REVISION" `
        -Arguments (Merge-Arguments $base @{ ExpectRevision = "deadbee" })
    Invoke-Case -Name "-ExpectBench names another fingerprint" -Expect "WRONG FINGERPRINT" `
        -Arguments (Merge-Arguments $base @{ ExpectBench = @(7185678, 1) })
    Invoke-Case -Name "-ExpectBench matches both sidecars" -Expect "" `
        -Arguments (Merge-Arguments $base @{ ExpectBench = @(7185678, 6199302) })

    Invoke-Case -Name "tune build entered in a gate" -Expect "tune build" `
        -Arguments (Merge-Arguments $base @{ EngineA = $tune.Engine; NameA = "tuneArm" })

    # A categorical A/B on one tune build waives the PGO-gate rule, and only
    # then: same executable both sides, differing by options, match mode.
    $catTune = Copy-Arm -Source $TuneEngine -Name "catTune"
    $categorical = @{
        Mode = "match"; EngineA = $catTune; EngineB = $catTune; NameA = "alt"; NameB = "base"
        OptionsA = @("CoreRfpLinear=30"); Games = 200; Seed = 7; CategoricalTuneBuild = $true
        Dir = (Join-Path $scratch "categorical"); DryRun = $true; AllowBusyHost = $true
    }
    Invoke-Case -Name "control: a categorical on one tune build" -Expect "" -Arguments $categorical
    Invoke-Case -Name "categorical waiver with two binaries" -Expect "ONE binary" `
        -Arguments (Merge-Arguments $categorical @{ EngineB = $armB; NameB = "armB" })
    Invoke-Case -Name "categorical waiver in a gate" -Expect "match only" `
        -Arguments (Merge-Arguments $categorical @{ Mode = "sprt"; MaxPairs = 100 } -Remove @("Games"))
    Invoke-Case -Name "categorical waiver with equal options" -Expect "differ by UCI options" `
        -Arguments (Merge-Arguments $categorical @{ OptionsA = @() })

    $otherCompiler = Copy-Arm -Source $EngineB -Name "otherCompiler"
    Set-Sidecar -EnginePath $otherCompiler -Fields @{ rustc = "rustc 1.97.0 (not the pinned toolchain)" }
    Invoke-Case -Name "arms built by different compilers" -Expect "COMPILER MISMATCH" `
        -Arguments (Merge-Arguments $base @{ EngineB = $otherCompiler; NameB = "otherCompiler" })

    $otherFlavor = Copy-Arm -Source $EngineB -Name "otherFlavor"
    Set-Sidecar -EnginePath $otherFlavor -Fields @{ flavor = "pext" }
    Invoke-Case -Name "arms built to different flavours" -Expect "BUILD FLAVOR MISMATCH" `
        -Arguments (Merge-Arguments $base @{ EngineB = $otherFlavor; NameB = "otherFlavor" })

    Invoke-Case -Name "one binary, one question, no options" -Expect "require -Mode calibrate" `
        -Arguments (Merge-Arguments $base @{ EngineB = $armA; NameB = "armAagain" })
    Invoke-Case -Name "null pair of two different binaries" -Expect "byte-identical" `
        -Arguments @{ Mode = "calibrate"; EngineA = $armA; EngineB = $armB; NameA = "a"; NameB = "b"
                      Dir = (Join-Path $scratch "null"); DryRun = $true; AllowBusyHost = $true }

    Invoke-Case -Name "option the engine does not expose" -Expect "does not advertise" `
        -Arguments (Merge-Arguments $base @{ OptionsA = @("NoSuchOption=1") })

    Invoke-Case -Name "a parameter the mode cannot honour" -Expect "ignores" `
        -Arguments (Merge-Arguments $base @{ Games = 500 })

    Invoke-Case -Name "conditions off Rarog's policy" -Expect "not Rarog's policy" `
        -Arguments (Merge-Arguments $base @{ Hash = 128 })

    # The runner pin, checked by pointing the wrapper at a pin file whose hash is
    # not the staged binary's.
    $wrongPin = Join-Path $scratch "wrong.pin.json"
    $pinData = Get-Content -LiteralPath $pin -Raw | ConvertFrom-Json
    $pinData.sha256 = ("A" * 64)
    $pinData | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $wrongPin -Encoding utf8
    Invoke-Case -Name "runner is not the pinned build" -Expect "COLOSSEUM PIN MISMATCH" `
        -Arguments (Merge-Arguments $base @{ PinPath = $wrongPin })

    # The idle guard, both branches. The CPU branch is forced with a ceiling no
    # host can be under; the process branch is forced by actually starting an
    # engine, because a guard that has only been reasoned about is not proven.
    Invoke-Case -Name "host CPU over the ceiling" -Expect "HOST NOT IDLE" `
        -Arguments (Merge-Arguments $base @{ MaxHostBusyPercent = -1 } @("AllowBusyHost"))

    $probePath = Join-Path $scratch "rarog-guardprobe.exe"
    Copy-Item -LiteralPath (Join-Path $repo $EngineA) -Destination $probePath -Force
    $psi = [System.Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = $probePath
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $psi.UseShellExecute = $false
    # Nothing is ever written to its stdin, so it waits for input and stays alive.
    $probe = [System.Diagnostics.Process]::Start($psi)
    try {
        Start-Sleep -Milliseconds 300
        Invoke-Case -Name "an engine process is already running" -Expect "HOST NOT IDLE" `
            -Arguments (Merge-Arguments $base @{} @("AllowBusyHost"))
    } finally {
        if (-not $probe.HasExited) { $probe.Kill($true) }
        $probe.Dispose()
    }

    $offArm = Copy-Arm -Source $OffArmTuneEngine -Name "offArm"
    Set-Sidecar -EnginePath $offArm -Fields @{ git_dirty = $false }
    Invoke-Case -Name "Core* surface on an off-arm tune build" -Expect "selectivity-core options" `
        -Arguments (Merge-Arguments $tune @{ Engine = $offArm })

    # At N = 1e8 the smallest registered step (3) perturbs by 0.458, which rounds
    # to zero on an integer option: the coordinate would look tuned and move
    # nothing.
    Invoke-Case -Name "tune horizon the surface cannot resolve" -Expect "rounds to zero" `
        -Arguments (Merge-Arguments $tune @{ Iterations = 100000000; TotalGames = 3000000 })

    Invoke-Case -Name "control: the registered tune resolves" -Expect "" -Arguments $tune

    # A gauntlet resolves through `tournament run`, whose dry run names the
    # command and its clock differently from the two-arm commands; the first
    # gauntlet through the wrapper was refused by its own policy check.
    $field = @('--format', 'gauntlet', '--seeds', '1', '--games-per-pair', '2', '--cycles', '1',
               '--engine', $armA, '--label', 'A', '--engine', $armB, '--label', 'B',
               '--rating', '3000', '--rating', '3000', '--fixed', '2:3000', '--option', 'Threads=1')
    $gauntlet = @{
        Mode = "gauntlet"; Seed = 7; Dir = (Join-Path $scratch "gauntlet"); DryRun = $true; AllowBusyHost = $true
        ExtraArgs = @($field + @('--option', 'Hash=64'))
    }
    Invoke-Case -Name "control: a gauntlet resolves" -Expect "" -Arguments $gauntlet
    Invoke-Case -Name "gauntlet participants off the Hash policy" -Expect "not Rarog's policy" `
        -Arguments (Merge-Arguments $gauntlet @{ ExtraArgs = @($field + @('--option', 'Hash=128')) })

    # The post-run fault guard, on status views in the shape cli-v0.1.0's
    # `status --json` returns. Its first live run found the earlier guard
    # parsing progress text it did not recognise and passing the run with a
    # warning, so each branch is watched here.
    . (Join-Path $repo "tools\harness_common.ps1")
    function New-Status([hashtable]$Checkpoint) {
        [pscustomobject]@{ durable = [pscustomobject]@{ checkpoint = $(if ($Checkpoint) { [pscustomobject]$Checkpoint }) } }
    }
    function Sides([int]$EngineA = 0, [int]$EngineB = 0, [int]$TimeA = 0, [int]$TimeB = 0, [int]$Infra = 0) {
        @{ faults = [pscustomobject]@{ engine_a = $EngineA; engine_b = $EngineB; time_losses_a = $TimeA
                                       time_losses_b = $TimeB; infrastructure = $Infra } }
    }
    function Invoke-FaultCase([string]$Name, [string]$Expect, [object]$Status) {
        $check = { Assert-ColosseumRunFaults -Status $Status -ScoredGames 200 `
                       -TimeLossRateCeiling 0.5 -Dir "scratch" *>&1 | Out-Null }.GetNewClosure()
        Invoke-Case -Name $Name -Expect $Expect -Check $check
    }
    Invoke-FaultCase "faults: a crash on side B" "non-time engine fault" (New-Status (Sides -EngineB 1))
    Invoke-FaultCase "faults: an infrastructure fault" "non-time engine fault" (New-Status (Sides -Infra 1))
    Invoke-FaultCase "faults: time losses over the ceiling" "exceeds the 0.5% ceiling" `
        (New-Status (Sides -EngineA 1 -EngineB 1 -TimeA 1 -TimeB 1))
    Invoke-FaultCase "faults: no checkpoint" "FAULT COUNTERS UNREADABLE" (New-Status $null)
    Invoke-FaultCase "faults: a field missing" "FAULT COUNTERS UNREADABLE" `
        (New-Status @{ faults = [pscustomobject]@{ engine_a = 0; engine_b = 0 } })
    Invoke-FaultCase "faults: a tournament fault" "does not separate time losses" (New-Status @{ engine_faults = 1 })
    Invoke-FaultCase "control: a clean run" "" (New-Status (Sides))
    Invoke-FaultCase "control: one time loss under the ceiling" "" (New-Status (Sides -EngineB 1 -TimeB 1))
    Invoke-FaultCase "control: a clean tournament" "" (New-Status @{ engine_faults = 0 })

    # Exit codes carry the verdict: H0 and a cap stop are finished runs to be
    # checked, not failures, and an invalid or cancelled run is never a verdict.
    $exitTable = @(
        @("sprt", 0, "outcome", "H1 accepted"), @("sprt", 1, "outcome", "H0 accepted")
        @("sprt", 4, "outcome", "cap reached, inconclusive"), @("sprt", 5, "invalid", $null)
        @("calibrate", 1, "outcome", "fail"), @("calibrate", 4, "outcome", "inconclusive")
        @("match", 0, "outcome", "completed"), @("match", 1, "invalid", $null)
        @("spsa", 5, "invalid", $null), @("gauntlet", 1, "invalid", $null)
        @("spsa", 6, "cancelled", $null), @("match", 2, "refused", $null), @("sprt", 3, "error", $null)
    )
    Invoke-Case -Name "exit codes map to the runner's verdicts" -Expect "" -Check {
        foreach ($row in $exitTable) {
            $resolved = Resolve-ColosseumExit -Mode $row[0] -ExitCode $row[1]
            if ($resolved.Kind -ne $row[2] -or $resolved.Verdict -ne $row[3]) {
                throw "$($row[0]) exit $($row[1]) resolved to $($resolved.Kind)/$($resolved.Verdict)"
            }
        }
    }.GetNewClosure()

    # Resuming an existing run directory: its recorded seed is carried over, and
    # a different -Seed is refused before anything runs.
    $resumeDir = Join-Path $scratch "resume"
    New-Item -ItemType Directory -Path $resumeDir | Out-Null
    '{"master_seed": 4242}' | Set-Content -LiteralPath (Join-Path $resumeDir "resolved-config.json") -Encoding utf8
    Invoke-Case -Name "resume under a different -Seed" -Expect "would be refused on resume" `
        -Arguments (Merge-Arguments $base @{ Dir = $resumeDir; Seed = 7 })
    $resumeArguments = Merge-Arguments $base @{ Dir = $resumeDir } @("Seed")
    Invoke-Case -Name "control: resume at the recorded seed" -Expect "" -Check {
        $output = & $wrapper @resumeArguments *>&1 | Out-String
        if ($output -notmatch 'at its recorded seed 4242' -or $output -notmatch 'seed 4242\)') {
            throw "the recorded seed 4242 was not carried into the run"
        }
    }.GetNewClosure()
} finally {
    if (-not $KeepScratch) { Remove-Item -LiteralPath $scratch -Recurse -Force -ErrorAction SilentlyContinue }
}

$failed = @($results | Where-Object { -not $_.Passed })
Write-Host ""
Write-Host ("{0}/{1} guard cases behaved." -f ($results.Count - $failed.Count), $results.Count)
if ($failed.Count -gt 0) {
    foreach ($case in $failed) { Write-Host "  FAILED: $($case.Name) - $($case.Detail)" -ForegroundColor Red }
    exit 1
}
exit 0
