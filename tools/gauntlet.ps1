<#
.SYNOPSIS
    Run the Rarog 2.2.0 end-of-Phase-4 external gauntlet via fastchess.

.DESCRIPTION
    Runs fastchess in gauntlet mode: Rarog 2.2.0 (the candidate) plays every
    other engine in the field, but the field engines do not play each other.
    This is the real-opponent transfer check for the staged Phase 4 self-play
    SPRT campaign (see PLAN.md SS10) -- self-play SPRT systematically
    overstates strength against a diverse field.

    Engine list mirrors D:\chess\little blitzer\engines_fast.lbe (same
    binaries, same UCI_Elo caps for the Stockfish entries).

    Uses a generous -TimeMargin (default 1000 ms) because Critter 1.6a (and
    other old single-threaded engines) have been observed to forfeit on time
    heavily under Little Blitzer's GUI-side clock at 10+0.1, even though they
    are not actually overstepping the clock -- see the recorded
    "Little Blitzer time confound" issue. fastchess measures engine process
    time directly rather than polling like a GUI, so it is expected to be far
    more reliable here; the wide margin is extra insurance, not a crutch.

.PARAMETER TC
    Clock time control string, fastchess "tc=" syntax. Default "10+0.1".

.PARAMETER Rounds
    Number of gauntlet rounds (each round = -games games per Rarog/opponent
    pair). Default 150 -> with -games 2 that's 300 games per opponent
    (9 opponents -> 2700 games total).

.PARAMETER TimeMargin
    fastchess timeout margin in milliseconds. Default 1000 (much more
    generous than the 20ms used for internal self-play SPRT) specifically to
    avoid false time forfeits for old/slow-IO engines like Critter and Fruit.

.PARAMETER Concurrency
    Parallel games. Default 8 (lower than self-play SPRT's higher
    concurrency) to reduce scheduling jitter for the time-sensitive old
    engines.

.PARAMETER Seed
    Opening randomization seed. Default 0 generates and reports a seed.

.PARAMETER Hash
    Hash size in MB for every engine. Default 64.

.PARAMETER Book
    Opening book, PGN or EPD (format auto-detected from the extension).
    Default tools\books\UHO_Lichess_4852_v1.epd; use IM_4mvs.pgn for a
    CCRL-comparable balanced-book run.

.PARAMETER FastchessPath
    Path to fastchess.exe. Default tools\bin\fastchess.exe (or found on PATH).

.PARAMETER EnginesDir
    Directory containing the opponent binaries. Default D:\chess\engines.

.EXAMPLE
    pwsh tools\gauntlet.ps1
    pwsh tools\gauntlet.ps1 -TC "30+0.3" -Rounds 60   # slower CCRL-anchor pass
#>
param(
    [string]$TC = "10+0.1",
    [int]$Rounds = 150,
    [int]$TimeMargin = 1000,
    [int]$Concurrency = 8,
    [int]$Seed = 0,
    [int]$Hash = 64,
    [string]$Book = "$PSScriptRoot\books\UHO_Lichess_4852_v1.epd",
    [string]$FastchessPath = "$PSScriptRoot\bin\fastchess.exe",
    [string]$EnginesDir = "D:\chess\engines"
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "harness_common.ps1")

$strengthProfile = Get-StrengthTestProfile
$resignArgs = @(Get-StrengthTestResignArgs)

$concurrencyInfo = Resolve-HarnessConcurrency -Requested $Concurrency
$Concurrency = $concurrencyInfo.Concurrency
$AffinityCpus = Get-HarnessAffinityCpuList -Concurrency $Concurrency
$Seed = New-HarnessSeed -Requested $Seed

# Locate fastchess.
$fastchess = $FastchessPath
if (-not (Test-Path $fastchess)) {
    $onPath = Get-Command fastchess -ErrorAction SilentlyContinue
    if ($onPath) { $fastchess = $onPath.Source }
    else {
        throw "fastchess not found at '$FastchessPath' or on PATH. Download from " +
              "https://github.com/Disservin/fastchess/releases and place it there."
    }
}
if (-not (Test-Path $Book)) { throw "Not found: $Book" }
$Book = (Resolve-Path $Book).Path
$fcInfo = Assert-AffinityFastchess -Path $fastchess

# Book format auto-detected from the extension (.epd -> format=epd, else pgn).
# NOTE: gauntlet numbers on the UHO book are for RELATIVE progress (release vs
# prior release / siblings). For a CCRL-comparable absolute estimate, pass
# -Book "$PSScriptRoot\books\IM_4mvs.pgn" — CCRL uses balanced openings.
$bookFormat = if ([System.IO.Path]::GetExtension($Book) -ieq ".epd") { "epd" } else { "pgn" }

# Candidate (gauntlet seed) + field, mirroring engines_fast.lbe.
$rarog22 = Join-Path $EnginesDir "rarog-v2.2.0-windows-pext-pgo.exe"
$sf      = Join-Path $EnginesDir "stockfish-windows-x86-64-bmi2.exe"
$rarog21 = Join-Path $EnginesDir "rarog-v2.1.0-windows-pext-pgo.exe"
$rarog20 = Join-Path $EnginesDir "rarog-v2.0.2-windows-pext-pgo.exe"
$bas16   = Join-Path $EnginesDir "basilisk-v1.6.0-windows-x86_64-pext-pgo.exe"
$bas15   = Join-Path $EnginesDir "basilisk-v1.5.0-windows-x86_64-pext-pgo.exe"
$critter = Join-Path $EnginesDir "Critter_1.6a_64bit.exe"
$fruit   = Join-Path $EnginesDir "fruit_21.exe"

foreach ($p in @($rarog22, $sf, $rarog21, $rarog20, $bas16, $bas15, $critter, $fruit)) {
    if (-not (Test-Path $p)) { throw "Not found: $p" }
}

$resultsDir = Join-Path $PSScriptRoot "results"
New-Item -ItemType Directory -Force -Path $resultsDir | Out-Null
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$pgnOut    = Join-Path $resultsDir "gauntlet_rarog220_${timestamp}.pgn"
$logOut    = [System.IO.Path]::ChangeExtension($pgnOut, ".log")

Write-Host ""
Write-Host "======================================================="
Write-Host "  GAUNTLET: Rarog 2.2.0  vs  field (9 opponents)"
Write-Host "  TC: tc=$TC   Margin: ${TimeMargin} ms   Hash: ${Hash} MB   Conc: $Concurrency"
Write-Host "  Rounds: $Rounds (games=2 -> $($Rounds * 2) games/opponent, $($Rounds * 2 * 9) total)"
Write-Host "  Book: $(Split-Path $Book -Leaf)"
Write-Host "  Seed: $Seed   CPUs: $AffinityCpus"
Write-Host "  Runner: $($fcInfo.Text)"
Write-Host "  PGN:  $pgnOut"
Write-Host "======================================================="
Write-Host ""

& $fastchess `
    -engine "cmd=$rarog22" "name=Rarog 2.2.0" "option.Hash=$Hash" "option.Threads=1" `
    -engine "cmd=$sf" "name=Stockfish 18-2900" "option.Hash=$Hash" "option.Threads=1" "option.UCI_LimitStrength=true" "option.UCI_Elo=2900" `
    -engine "cmd=$sf" "name=Stockfish 18-2800" "option.Hash=$Hash" "option.Threads=1" "option.UCI_LimitStrength=true" "option.UCI_Elo=2800" `
    -engine "cmd=$sf" "name=Stockfish 18-2700" "option.Hash=$Hash" "option.Threads=1" "option.UCI_LimitStrength=true" "option.UCI_Elo=2700" `
    -engine "cmd=$rarog21" "name=Rarog 2.1.0" "option.Hash=$Hash" "option.Threads=1" `
    -engine "cmd=$rarog20" "name=Rarog 2.0.2" "option.Hash=$Hash" "option.Threads=1" `
    -engine "cmd=$bas16" "name=Basilisk 1.6.0" "option.Hash=$Hash" "option.Threads=1" `
    -engine "cmd=$bas15" "name=Basilisk 1.5.0" "option.Hash=$Hash" "option.Threads=1" `
    -engine "cmd=$critter" "name=Critter 1.6a" "option.Hash=$Hash" "option.Threads=1" `
    -engine "cmd=$fruit" "name=Fruit 2.1" "option.Hash=$Hash" "option.Threads=1" `
    -each "tc=$TC" "timemargin=$TimeMargin" `
    -openings "file=$Book" "format=$bookFormat" order=random `
    -tournament gauntlet -rounds $Rounds -games 2 -repeat `
    -concurrency $Concurrency `
    -use-affinity $AffinityCpus `
    -srand $Seed `
    -draw "movenumber=$($strengthProfile.DrawMoveNumber)" "movecount=$($strengthProfile.DrawMoveCount)" "score=$($strengthProfile.DrawScore)" `
    @resignArgs `
    -pgnout "file=$pgnOut" `
    -output format=fastchess 2>&1 |
    Tee-Object -FilePath $logOut

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Error "fastchess exited with code $LASTEXITCODE — no games were played."
} else {
    Assert-NoAffinityFailure -LogPath $logOut
    Write-Host ""
    Write-Host "Gauntlet finished. PGN: $pgnOut"
    Write-Host "Next: ordo-win64.exe -p `"$pgnOut`" -o ratings.txt -a 2780 -A `"Fruit 2.1`""
}
