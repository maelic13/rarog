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
# A tune reads the rule 7c conversion of its surface beside its run file. The
# suite generates one for b23core inside the ignored tools/results, where the
# run file's relative path to the tune policy resolves, and checks it there.
$surfaceScratch = Join-Path $repo ("tools\results\guards-surface-" + [guid]::NewGuid())
python (Join-Path $repo "tools\spsa_config_to_colosseum.py") b23core --rule7c --ceiling 6000 --out $surfaceScratch | Out-Null
if ($LASTEXITCODE -ne 0) { throw "Could not generate the rule 7c surface the tune cases need." }

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
        ConfigGroup = "b23core"; Ceiling = 6000; Seed = 7
        RunFile = (Join-Path $surfaceScratch "b23core.run.toml")
        Dir = (Join-Path $scratch "tune"); DryRun = $true; AllowBusyHost = $true
    }

    Write-Host ""
    Write-Host "Guard cases (dry runs; no game is played):"

    Invoke-Case -Name "control: a clean gate resolves" -Expect "" -Arguments $base

    # A multi-thread match must give each game a physical core per thread.
    $fourThreads = Merge-Arguments $base @{
        Mode = "match"; Games = 100; Threads = 4; Hash = 512; Concurrency = 3
        OptionsA = @("Threads=4"); OptionsB = @("Threads=4")
    } @("MaxPairs")
    Invoke-Case -Name "four threads on one core a game" -Expect "cores_per_game is 1 for Threads 4" `
        -Arguments $fourThreads
    Invoke-Case -Name "control: four threads, four cores" -Expect "" `
        -Arguments (Merge-Arguments $fourThreads @{ ExtraArgs = @("--cores-per-game", "4") })

    # A self-relative scaling match: one engine at one thread against itself at
    # four. The larger side sets the cores a game needs.
    $scaling = Merge-Arguments $base @{
        Mode = "match"; Games = 100; Threads = 1; ThreadsB = 4; Hash = 512; Concurrency = 3
        EngineB = $armA; NameB = "armA4t"
    } @("MaxPairs")
    Invoke-Case -Name "arm B's four threads on one core" -Expect "cores_per_game is 1 for Threads 4" `
        -Arguments $scaling
    Invoke-Case -Name "control: one binary, 1T against 4T" -Expect "" `
        -Arguments (Merge-Arguments $scaling @{ ExtraArgs = @("--cores-per-game", "4") })
    # A time-odds match: one engine at twice the time against itself.
    $odds = Merge-Arguments $base @{
        Mode = "match"; Games = 100; EngineB = $armA; NameB = "armA2x"; BaseMsB = 6000; IncrementMsB = 60
    } @("MaxPairs")
    Invoke-Case -Name "control: one binary, 1x against 2x" -Expect "" -Arguments $odds
    # -Hash with no side options must still reach both engines; the policy
    # check reads each side's resolved Hash.
    Invoke-Case -Name "control: -Hash alone reaches both sides" -Expect "" `
        -Arguments (Merge-Arguments $odds @{ Hash = 256 })
    Invoke-Case -Name "time odds on a fixed-node read" -Expect "a fixed-node read (-Nodes) has none" `
        -Arguments (Merge-Arguments $odds @{ Nodes = 5000 })
    Invoke-Case -Name "-BaseMsB in a null pair" -Expect "-Mode calibrate ignores: BaseMsB" `
        -Arguments @{ Mode = "calibrate"; EngineA = $armA; EngineB = $armA; NameA = "a"; NameB = "b"; BaseMsB = 6000
                      Dir = (Join-Path $scratch "null"); DryRun = $true; AllowBusyHost = $true }
    Invoke-Case -Name "-ThreadsB in a null pair" -Expect "-Mode calibrate ignores: ThreadsB" `
        -Arguments @{ Mode = "calibrate"; EngineA = $armA; EngineB = $armA; NameA = "a"; NameB = "b"; ThreadsB = 4
                      Dir = (Join-Path $scratch "null"); DryRun = $true; AllowBusyHost = $true }

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

    # The arms' names are the runner's labels; with one executable on both sides
    # they are all that keeps the two sides' statistics apart.
    Invoke-Case -Name "two arms with one name" -Expect "need distinct names" `
        -Arguments (Merge-Arguments $categorical @{ NameB = "ALT" })
    Invoke-Case -Name "a second label in -ExtraArgs" -Expect "'--a-label <A_LABEL>' cannot be used multiple times" `
        -Arguments (Merge-Arguments $categorical @{ ExtraArgs = @("--a-label", "impostor") })

    # -Nodes: a fixed-node read resolves fixed nodes on both sides and nothing
    # else; the clock run file under -Nodes, or -Nodes outside a match, is
    # refused before any game.
    Invoke-Case -Name "control: a fixed-node read resolves" -Expect "" `
        -Arguments (Merge-Arguments $categorical @{ Nodes = 150000 })
    # The CLI itself refuses two time controls on one side; the wrapper relays
    # its reason.
    Invoke-Case -Name "-Nodes with the clock run file" -Expect "only one of movetime, base/increment, nodes" `
        -Arguments (Merge-Arguments $categorical @{ Nodes = 150000; RunFile = (Join-Path $repo "tools\colosseum\match-fixed.toml") })
    Invoke-Case -Name "-Nodes in a gate" -Expect "-Nodes is for -Mode match only" `
        -Arguments (Merge-Arguments $base @{ Nodes = 150000 })
    Invoke-Case -Name "fixed-node run file without -Nodes" -Expect "expected the clock" `
        -Arguments (Merge-Arguments $categorical @{ RunFile = (Join-Path $repo "tools\colosseum\match-fixed-nodes.toml") })

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

    $fixtureTables = Join-Path $repo "tests\fixtures\syzygy"
    $emptyTables = Join-Path $scratch "no-tables"
    New-Item -ItemType Directory -Path $emptyTables | Out-Null
    Invoke-Case -Name "control: the same tables on both sides" -Expect "" `
        -Arguments (Merge-Arguments $base @{ OptionsA = @("SyzygyPath=$fixtureTables")
                                              OptionsB = @("SyzygyPath=$fixtureTables") })
    Invoke-Case -Name "tables on one side only" -Expect "SyzygyPath differs" `
        -Arguments (Merge-Arguments $base @{ OptionsA = @("SyzygyPath=$fixtureTables") })
    Invoke-Case -Name "a tablebase folder with no tables" -Expect "holds no .rtbw" `
        -Arguments (Merge-Arguments $base @{ OptionsA = @("SyzygyPath=$emptyTables")
                                              OptionsB = @("SyzygyPath=$emptyTables") })

    # A runner argument the wrapper did not ask for moves the resolved
    # configuration off what it was told; the policy check reads the result.
    Invoke-Case -Name "conditions off Rarog's policy" -Expect "engine_b_time_control base is 4000 ms, expected 3000" `
        -Arguments (Merge-Arguments $base @{ ExtraArgs = @("--b-base-ms", "4000") })

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
    Invoke-Case -Name "tune ceiling the surface cannot resolve" -Expect "rounds to zero" `
        -Arguments (Merge-Arguments $tune @{ Ceiling = 100000000 })

    Invoke-Case -Name "control: the registered tune resolves" -Expect "" -Arguments $tune

    # PLAN rule 7c: one run to a registered ceiling, read at checkpoints.
    Invoke-Case -Name "a tune without a ceiling" -Expect "-Ceiling is required" `
        -Arguments (Merge-Arguments $tune -Remove @("Ceiling"))
    Invoke-Case -Name "a tune off the production shape" -Expect "production shape" `
        -Arguments (Merge-Arguments $tune @{ FirstCheckpoint = 1000 })
    Invoke-Case -Name "a tune on 14 slots" -Expect "production shape" `
        -Arguments (Merge-Arguments $tune @{ Concurrency = 14 })
    Invoke-Case -Name "a surface converted for another ceiling" -Expect "not the rule 7c conversion" `
        -Arguments (Merge-Arguments $tune @{ Ceiling = 5000 })
    Invoke-Case -Name "a block-form surface" -Expect "not the rule 7c conversion" `
        -Arguments (Merge-Arguments $tune -Remove @("RunFile"))
    Invoke-Case -Name "-Ceiling in a gate" -Expect "ignores" `
        -Arguments (Merge-Arguments $base @{ Ceiling = 6000 })
    Invoke-Case -Name "control: a short tune smoke resolves" -Expect "" `
        -Arguments (Merge-Arguments $tune @{ AllowShortRun = $true; FirstCheckpoint = 20; CheckpointEvery = 10
                                             Window = 20; GamesPerIteration = 4; Concurrency = 4 })

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

    # Every run keeps its engine processes; a command's default changed between
    # runner versions (a tournament started fresh ones before 0.3.0).
    Invoke-Case -Name "gauntlet with fresh engine processes" -Expect "engine processes are 'per-game'" `
        -Arguments (Merge-Arguments $gauntlet @{ ExtraArgs = @($field + @('--option', 'Hash=64', '--engine-processes', 'per-game')) })
    Invoke-Case -Name "gate with fresh engine processes" -Expect "engine processes are 'per-game'" `
        -Arguments (Merge-Arguments $base @{ ExtraArgs = @('--engine-processes', 'per-game') })

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

    # The runner's end-of-run results reach the manifest as its own figures
    # (run-record schema 6), and an older record says it has none.
    $schema6 = @'
{"schema_version": 6, "results": {
 "by_colour": [{"engine": "cand", "as_white": {"wins": 4, "draws": 8, "losses": 2, "games": 14, "score": 0.5714},
                "as_black": {"wins": 1, "draws": 8, "losses": 5, "games": 14, "score": 0.3571}}],
 "terminations": {"overall": {"Checkmate": 12, "Threefold": 12}},
 "faults": {"faults": [{"engine": "cand", "kind": "crash", "phase": "play", "termination": "Crash", "scored_as": "loss", "games": 1}],
            "time_losses": [{"engine": "cand", "games": 2}, {"engine": "base", "games": 0}], "infrastructure_faults": 0},
 "telemetry": {"engines": [{"engine": "cand", "annotated_moves": 1644,
   "depth": {"status": "available", "mean": 15.3966}, "non_mate_depth": {"status": "available", "mean": 13.889, "samples": 1559},
   "nodes": {"status": "available", "mean": 178234.7}, "elapsed_seconds": {"status": "available", "mean": 0.0719},
   "nodes_per_second": 2477303.07,
   "harness_overhead_ms": {"status": "available", "p50": 1.0, "p99": 9.0, "p999": 60.0, "max": 82.0, "over_margin": 4, "moves_with_margin": 1644}}]},
 "tournament": {"pairs": [{"engine": "cand", "opponent": "Houdini 3", "wins": 248, "draws": 88, "losses": 64, "games": 400,
                           "elo": 172.78, "elo_lower": 141.49, "elo_upper": 206.92, "pentanomial": [4, 6, 60, 62, 68]}],
                "standings": [{"rank": 1, "name": "cand", "rating": 3428.82, "error_95": 19.18, "fixed": false, "games": 2400},
                              {"rank": 2, "name": "Houdini 3", "rating": 3287.0, "error_95": null, "fixed": true, "games": 400}]}}}
'@ | ConvertFrom-Json
    $expectedLines = @(
        "by_colour:        cand: White 4 W / 8 D / 2 L of 14 (57.1%); Black 1 W / 8 D / 5 L of 14 (35.7%)"
        "terminations:     Checkmate 12, Threefold 12"
        "time_losses:      cand 2, base 0"
        "engine_faults:    cand crash in play, Crash, scored loss: 1"
        "infra_faults:     0"
        "search:           cand: 1644 moves; depth 15.40 mean; non-mate depth 13.89 over 1559 moves; 178235 nodes a move; 71.9 ms a move; 2477303 nps"
        "harness_hold_ms:  cand: p50 1, p99 9, p999 60, max 82; 4 of 1644 moves over the margin"
        "pair:             cand vs Houdini 3: 248 W / 88 D / 64 L of 400, Elo 172.8 [141.5, 206.9]; ptnml [4, 6, 60, 62, 68]"
        "standing:         1. cand 3428.8 +/- 19.2, 2400 games"
        "standing:         2. Houdini 3 3287.0 fixed, 400 games"
    )
    Invoke-Case -Name "results: schema 6 reaches the manifest" -Expect "" -Check {
        $got = @(Format-ColosseumResults -Record $schema6)
        if (($got -join "`n") -ne ($expectedLines -join "`n")) {
            throw "formatted results differ:`n$($got -join "`n")"
        }
    }.GetNewClosure()
    Invoke-Case -Name "results: a schema 5 record has none" -Expect "" -Check {
        $got = @(Format-ColosseumResults -Record ([pscustomobject]@{ schema_version = 5 }))
        if ($got.Count -ne 1 -or $got[0] -notlike "results:*not in this run record (schema 5)") { throw "got: $got" }
    }.GetNewClosure()
    Invoke-Case -Name "spsa: rail and clamp warnings recorded" -Expect "" -Check {
        $rail = Format-ColosseumRailWarnings -Warnings @([pscustomobject]@{
            name = "CoreRfpCorrection"; seed = 80; c0 = 128.0000776908343; min = 0; max = 2048
            clamped_at_min = $true; clamped_at_max = $false })
        if ($rail -ne "rail_warnings:    CoreRfpCorrection seed 80 c_0 128 -> min 0") { throw "rail: $rail" }
        if ((Format-ColosseumRailWarnings -Warnings @()) -ne "rail_warnings:    none") { throw "an empty list is not 'none'" }
        $result = '{"schema_version": 4, "tune_audit": {"warnings": [{"kind": "initial-on-lower-rail", "name": "X"},
            {"kind": "arms-clamped-at-range-end", "name": "Y", "min": 0, "max": 10, "completed_iterations": 50,
             "clamped_at_min": 7, "clamped_at_max": 0}]}}' | ConvertFrom-Json
        $clamp = Format-ColosseumClampWarnings -Result $result
        if ($clamp -ne "arm_clamps:       Y at min on 7, at max on 0 of 50 iterations") { throw "clamp: $clamp" }
        $old = Format-ColosseumClampWarnings -Result ('{"schema_version": 3, "tune_audit": {"warnings": []}}' | ConvertFrom-Json)
        if ($old -ne "arm_clamps:       not recorded (result schema 3)") { throw "a schema 3 audit read as: $old" }
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
    if (-not $KeepScratch) {
        Remove-Item -LiteralPath $scratch -Recurse -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $surfaceScratch -Recurse -Force -ErrorAction SilentlyContinue
    }
}

$failed = @($results | Where-Object { -not $_.Passed })
Write-Host ""
Write-Host ("{0}/{1} guard cases behaved." -f ($results.Count - $failed.Count), $results.Count)
if ($failed.Count -gt 0) {
    foreach ($case in $failed) { Write-Host "  FAILED: $($case.Name) - $($case.Detail)" -ForegroundColor Red }
    exit 1
}
exit 0
