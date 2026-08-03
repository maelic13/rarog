<#
.SYNOPSIS
    Generate a deterministic self-play PGN segment for Texel tuning.

.DESCRIPTION
    Runs fastchess self-play between two copies of the given engine at a fixed
    node limit, collecting a large PGN file suitable for tools\texel\extract.py.

    A fixed -Seed shuffles the opening book reproducibly. -Start and -Rounds
    select a non-wrapping range in that shuffled order, so a pilot and its
    continuation cannot silently replay openings. The output filename records
    the engine suffix, node count, start, and game count.

    Adjudication uses the named datagen-v1 profile: draw after move 40 with an
    8-move window at score < 10 cp; resign after 3 moves at score > 600 cp only
    when both engines agree. This is deliberately stricter than strength-v1:
    one wrong game result would mislabel many training positions.

.PARAMETER Suffix
    Engine binary suffix. Looks for
    tools\test_engines\rarog-<Suffix>-pext-pgo.exe.
    Build with:  .\tools\build_test.ps1 -Suffix <Suffix>

.PARAMETER Rounds
    Number of games. Default 0 consumes the unused tail from -Start through the
    final book entry. A range that would wrap around the book is rejected.

.PARAMETER Start
    One-based index into the book after its deterministic shuffle. Default 1.
    Use Start=20001 for the continuation after a 20,000-game pilot.

.PARAMETER Seed
    fastchess opening-shuffle seed. Keep it identical across segments that are
    intended to partition one book. Default 10403 (Phase 10.4.3).

.PARAMETER Nodes
    Node limit per move. Default 8000 (fast, diverse). Values 5000-12000 add
    variety; combine multiple runs with different nodes for the train split.

.PARAMETER Hash
    Hash table size per engine in MB. Default 16 (small enough to keep per-game
    state mostly cache-hot at this node count).

.PARAMETER Concurrency
    Parallel games. Default: physical CPU count minus 2, which leaves the PC
    usable. An explicit higher value is allowed for maximum throughput because
    fixed-node games remain deterministic under oversubscription.

.PARAMETER OutputPgn
    Path for the output PGN file. Existing files are never overwritten.
    Default includes suffix, nodes, start, and games.

.PARAMETER Append
    Obsolete safety trap. Appending is rejected because it destroys the
    one-segment/one-manifest provenance contract; extraction accepts many PGNs.

.PARAMETER SetupOnly
    Validate all inputs and provenance, print the exact command and segment,
    then exit without starting any games or creating output files.

.PARAMETER Book
    Opening book PGN/EPD. Default: tools\texel\data\beast_seed.epd (diverse,
    for training-position yield — NOT the unbalanced UHO SPRT book). When this
    default is used, -BookFormat defaults to epd unless you pass it explicitly.

.PARAMETER BookFormat
    Opening book format passed to fastchess: pgn or epd. Default: pgn.

.PARAMETER FastchessPath
    Path to fastchess.exe. Default: tools\bin\fastchess.exe

.EXAMPLE
    # First measure a 20k pilot.
    .\tools\datagen.ps1 -Suffix p1025a-zero -Rounds 20000 -Start 1 -Seed 10403

.EXAMPLE
    # If preflight recommends 180k total, generate exactly the disjoint tail.
    .\tools\datagen.ps1 -Suffix p1025a-zero -Rounds 160000 -Start 20001 -Seed 10403
#>
param(
    [Parameter(Mandatory)][string]$Suffix,
    [int]   $Rounds      = 0,         # 0 = consume the tail beginning at Start
    [int]   $Start       = 1,
    [int]   $Seed        = 10403,
    [int]   $Nodes       = 8000,
    [int]   $Hash        = 16,
    [int]   $Concurrency = 0,        # 0 = auto (physical CPUs - 2)
    [string]$OutputPgn   = "",
    [string]$Book        = "",
    [ValidateSet("pgn", "epd")]
    [string]$BookFormat  = "pgn",
    [string]$FastchessPath = "",
    [switch]$Append,
    [switch]$SetupOnly
)

$ErrorActionPreference = "Stop"
. "$PSScriptRoot\harness_common.ps1"

function Get-TextLineCount([string]$Path) {
    $reader = [System.IO.File]::OpenText($Path)
    try {
        $count = 0
        while ($null -ne $reader.ReadLine()) { $count++ }
        return $count
    } finally {
        $reader.Dispose()
    }
}

$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot

try {
    # ---- Defaults resolved relative to repo root ----
    # Datagen deliberately does NOT use the UHO SPRT/SPSA book: UHO openings are
    # curated to a ~+0.5-pawn White edge, which would bias the training-position
    # distribution. Training data wants DIVERSE, representative coverage, so the
    # default is the current 750k phase-balanced beast_seed.epd; the diversity
    # guard below enforces a non-wrapping independent-opening range. Override
    # -Book only for a deliberate experiment.
    if (-not $Book) {
        $Book = "$PSScriptRoot\texel\data\beast_seed.epd"
        if (-not $PSBoundParameters.ContainsKey('BookFormat')) { $BookFormat = "epd" }
    }
    if (-not $FastchessPath) { $FastchessPath = "$PSScriptRoot\bin\fastchess.exe" }
    $enginePath = "$PSScriptRoot\test_engines\rarog-$Suffix-pext-pgo.exe"

    foreach ($p in @($Book, $FastchessPath, $enginePath)) {
        if (-not (Test-Path $p)) { throw "Not found: $p" }
    }
    $enginePath   = (Resolve-Path $enginePath).Path
    $Book         = (Resolve-Path $Book).Path
    $FastchessPath = (Resolve-Path $FastchessPath).Path

    if ($Start -lt 1) { throw "Start must be >= 1 (got $Start)." }
    if ($Rounds -lt 0) { throw "Rounds must be >= 0 (got $Rounds)." }
    if ($Seed -lt 1) { throw "Seed must be >= 1 (got $Seed)." }
    if ($Nodes -lt 1) { throw "Nodes must be >= 1 (got $Nodes)." }
    if ($Hash -lt 1) { throw "Hash must be >= 1 MB (got $Hash)." }
    if ($Append) {
        throw "-Append is no longer supported: use a new disjoint -Start/-Rounds segment. extract.py accepts multiple PGNs."
    }

    # Auto concurrency leaves two physical cores for interactive use. Explicit
    # oversubscription is valid for deterministic fixed-node datagen.
    if ($Concurrency -le 0) {
        $Concurrency = [Math]::Max(1, (Get-PhysicalCoreCount) - 2)
    }

    # Book-diversity guard (Phase 6.2.0, lesson 5): fixed-node self-play from a
    # small book replays near-identical games — Basilisk got 31,880 unique
    # positions from 200k games off SuperGM_4mvs vs 1.73M off a diverse seed.
    if ($BookFormat -eq "epd") {
        $openings = Get-TextLineCount $Book
    } else {
        $openings = (Select-String -Path $Book -Pattern '^\[Event ' -SimpleMatch:$false).Count
    }
    if ($openings -le 0) { throw "Could not count openings in $Book." }
    if ($Start -gt $openings) {
        throw "Start $Start exceeds the $openings openings in the book."
    }
    $remaining = $openings - $Start + 1
    if ($Rounds -eq 0) { $Rounds = $remaining }
    if ($Rounds -gt $remaining) {
        throw "Segment [$Start, $($Start + $Rounds - 1)] exceeds the $openings-opening book and would wrap/reuse openings. Maximum Rounds from Start=$Start is $remaining."
    }

    $segmentEnd = $Start + $Rounds - 1
    if (-not $OutputPgn) {
        $OutputPgn = "$PSScriptRoot\texel\data\selfplay-$Suffix-n$Nodes-s$Start-g$Rounds.pgn"
    }
    $OutputPgn = [IO.Path]::GetFullPath($OutputPgn)

    if (Test-Path $OutputPgn) {
        throw "Output already exists: $OutputPgn. Choose a new segment or -OutputPgn; archives are never appended/overwritten."
    }

    $outputManifest = [IO.Path]::ChangeExtension($OutputPgn, ".manifest.json")
    if (Test-Path $outputManifest) {
        throw "Output manifest already exists: $outputManifest. Choose a new segment or -OutputPgn."
    }

    # Refuse anonymous or dirty label generators. The sidecar was produced by
    # build_test.ps1 after its bench smoke test.
    $engineManifestPath = [IO.Path]::ChangeExtension($enginePath, ".json")
    if (-not (Test-Path -LiteralPath $engineManifestPath)) {
        throw "Missing engine provenance manifest: $engineManifestPath. Rebuild with tools\build_test.ps1."
    }
    $engineManifest = Get-Content -LiteralPath $engineManifestPath -Raw | ConvertFrom-Json
    if ($engineManifest.engine -ne [IO.Path]::GetFileName($enginePath)) {
        throw "Engine manifest names '$($engineManifest.engine)', expected '$([IO.Path]::GetFileName($enginePath))'."
    }
    if ($engineManifest.git_dirty) {
        throw "Datagen engine was built from a dirty tree; rebuild a reproducible binary before generating labels."
    }

    $fastchessInfo = Get-FastchessVersion -Path $FastchessPath
    $profile = Get-DatagenProfile
    # Hash before launch so the manifest identifies the inputs fastchess
    # actually opened, even if a file is changed after the run begins.
    $engineHash = Get-HarnessSha256 -Path $enginePath
    $bookHash = Get-HarnessSha256 -Path $Book
    $resignArgs = @(Get-DatagenResignArgs)
    $fastchessArgs = @(
        '-engine', "cmd=$enginePath", 'name=A', "option.Hash=$Hash", 'option.Threads=1',
        '-engine', "cmd=$enginePath", 'name=B', "option.Hash=$Hash", 'option.Threads=1',
        '-each', 'tc=inf', "nodes=$Nodes",
        '-openings', "file=$Book", "format=$BookFormat", 'order=random', "start=$Start",
        '-srand', "$Seed",
        '-rounds', "$Rounds", '-games', '1',
        '-concurrency', "$Concurrency",
        '-draw', "movenumber=$($profile.DrawMoveNumber)", "movecount=$($profile.DrawMoveCount)", "score=$($profile.DrawScore)"
    ) + $resignArgs + @(
        '-pgnout', "file=$OutputPgn",
        '-output', 'format=fastchess'
    )

    $games = $Rounds
    Write-Host ""
    Write-Host "============================================================"
    Write-Host "  Rarog Texel datagen — self-play"
    Write-Host "  Engine  : $enginePath"
    Write-Host "  Games   : $games (one independent opening each)"
    Write-Host "  Segment : $Start..$segmentEnd of $openings (shuffled with seed $Seed)"
    Write-Host "  Nodes   : $Nodes per move"
    Write-Host "  Hash    : $Hash MB"
    Write-Host "  Conc.   : $Concurrency"
    Write-Host "  Book    : $(Split-Path $Book -Leaf) ($BookFormat)"
    Write-Host "  Book SHA: $bookHash"
    Write-Host "  Profile : $($profile.Name) (resign $($profile.ResignScore)/$($profile.ResignMoveCount), two-sided)"
    Write-Host "  Runner  : $($fastchessInfo.Text)"
    Write-Host "  Output  : $OutputPgn"
    Write-Host "============================================================"
    Write-Host ""

    # NOTE (2026-07-22): datagen deliberately has NO -use-affinity and may use
    # oversubscribed concurrency. Games are NODE-limited (tc=inf), so scheduler
    # placement cannot change a move or label; only throughput is affected.
    $quotedArgs = $fastchessArgs | ForEach-Object {
        if ($_ -match '[\s"]') { '"' + ($_ -replace '"', '\"') + '"' } else { $_ }
    }
    Write-Host "Command  : & `"$FastchessPath`" $($quotedArgs -join ' ')"

    if ($SetupOnly) {
        Write-Host ""
        Write-Host "SetupOnly: validation passed; no games or files were created."
        return
    }

    # SetupOnly is side-effect free; create the destination only for a real run.
    $outDir = Split-Path -Parent $OutputPgn
    if ($outDir -and -not (Test-Path $outDir)) {
        New-Item -ItemType Directory -Force -Path $outDir | Out-Null
    }

    & $FastchessPath @fastchessArgs

    if ($LASTEXITCODE -ne 0) {
        throw "fastchess exited with code $LASTEXITCODE."
    }

    Write-Host ""
    Write-Host "Done. PGN: $OutputPgn"

    $runManifest = [ordered]@{
        schema             = "rarog-datagen-v1"
        completed_utc      = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
        output_pgn         = $OutputPgn
        engine             = [ordered]@{
            path        = $enginePath
            sha256      = $engineHash
            git_sha     = $engineManifest.git_sha
            git_branch  = $engineManifest.git_branch
            git_dirty   = [bool]$engineManifest.git_dirty
            bench_nodes = [int64]$engineManifest.bench_nodes
            built_utc   = $engineManifest.built_utc
        }
        book               = [ordered]@{
            path      = $Book
            format    = $BookFormat
            sha256    = $bookHash
            openings  = $openings
            seed      = $Seed
            start     = $Start
            end       = $segmentEnd
        }
        games              = $Rounds
        nodes_per_move     = $Nodes
        hash_mb            = $Hash
        concurrency        = $Concurrency
        fastchess          = $fastchessInfo.Text
        adjudication       = $profile
    }
    $runManifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $outputManifest -Encoding utf8
    Write-Host "Manifest: $outputManifest"

    # Do not re-read a multi-GB PGN merely to count lines. The bounded preflight
    # below measures the quantity that matters: unique quiet yield per phase.
    Write-Host "Run extract.py --preflight-games 20000 on the 20k pilot before generating its continuation."
    Write-Host "The preflight sizes the exact total from measured limiting-phase unique yield."

} finally {
    Pop-Location
}
