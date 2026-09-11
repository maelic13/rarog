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

    Adjudication is OFF by default (profile datagen-v2, 2026-09-01, RAR-M17):
    games play to a rules result. The reason is sample depletion rather than
    mislabeling -- RAR-M15 measured adjudication ending 52.7% of all endgames
    before they are reached, which leaves an adjudicated corpus systematically
    short of the positions the endgame families need to be fitted on.

    Pass -Adjudicate for the legacy datagen-v1 profile, which `hce-v2` and
    every manifest written before 2026-09-01 used: draw after move 40 with an
    8-move window at score < 10 cp; resign after 3 moves at score > 600 cp only
    when both engines agree. Identical to strength-v2 since 2026-08-18, but
    kept as a separate named profile: one wrong game result mislabels every
    position sampled from that game, so labels must never silently follow a
    future loosening of the strength rule.

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
    One truncation remains and is deliberate: `-maxmoves 200`, a runaway guard
    that ends a game as a draw after 200 moves. It is not adjudication and it
    does not deplete the corpus the way the draw rule did -- measured against
    RAR-E06's 3,915 unadjudicated games, only 0.05% run past 400 plies, against
    the 52.7% of endgames the draw rule was ending. Keep it: without a cap a
    single pathological game can hold a concurrency slot indefinitely.

#>
param(
    [Parameter(Mandatory)][string]$Suffix,
    [int]   $Rounds      = 0,         # 0 = consume the tail beginning at Start
    [int]   $Start       = 1,
    [int]   $Seed        = 10403,
    [int]   $Nodes       = 8000,
    [int]   $Hash        = 16,
    [int]   $Concurrency = 0,        # 0 = auto (logical processors - 2; node-limited)
    [string]$OutputPgn   = "",
    [string]$Book        = "",
    [ValidateSet("pgn", "epd")]
    [string]$BookFormat  = "pgn",
    [string]$FastchessPath = "",
    [switch]$Append,
    # Opt back into datagen-v1 adjudication. Off by default since 2026-09-01.
    [switch]$Adjudicate,
    # Syzygy WDL directory. When set, games are adjudicated on TABLEBASE TRUTH
    # at 6 men (profile datagen-v3) instead of being played out at the datagen
    # node budget, which the engine often cannot convert. Never use this for a
    # strength gate. Mutually exclusive with -Adjudicate.
    [string]$SyzygyPath = "",
    [int]$SyzygyPieces = 6,
    [switch]$SetupOnly
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
. "$PSScriptRoot\harness_common.ps1"

function Get-UniqueEpdOpeningCount([string]$Path) {
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $count = 0
    foreach ($line in [IO.File]::ReadLines($Path)) {
        $parts = $line.Split(' ', [StringSplitOptions]::RemoveEmptyEntries)
        if ($parts.Count -lt 4) { throw "$Path opening $($count + 1) is not a four-field FEN." }
        $fen4 = $parts[0..3] -join ' '
        if (-not $seen.Add($fen4)) {
            throw "$Path repeats opening '$fen4'; refusing duplicate-seeded datagen."
        }
        $count++
    }
    return $count
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
        # Datagen is NODE-limited (`tc=inf nodes=N`), so it may oversubscribe
        # where a timed harness may not. A `go nodes` search plays identical
        # moves however slowly it runs: contention costs wall time and changes
        # nothing about the result. The physical-core rule that governs
        # sprt.ps1 exists for timed games, where contention causes the forfeits
        # RAR-M14 measured -- a hazard this path does not have. Defaulting to
        # physical-2 was leaving over half of a 32-thread machine idle.
        $Concurrency = (Resolve-HarnessConcurrency -Requested 0 -AllowOversubscribe).Concurrency
    }

    # Book-diversity guard (Phase 6.2.0, lesson 5): fixed-node self-play from a
    # small book replays near-identical games — Basilisk got 31,880 unique
    # positions from 200k games off SuperGM_4mvs vs 1.73M off a diverse seed.
    if ($BookFormat -eq "epd") {
        $openings = Get-UniqueEpdOpeningCount $Book
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
    $verificationProperty = $engineManifest.PSObject.Properties["verification"]
    if ($verificationProperty -and $verificationProperty.Value -ne "bench") {
        throw "Datagen engine manifest records '$($verificationProperty.Value)', not bench verification."
    }
    if ($engineManifest.flavor -like "*-tune") {
        throw "Datagen requires a production PGO build, not a tune binary."
    }

    $fastchessInfo = Get-FastchessVersion -Path $FastchessPath
    if ($Adjudicate -and $SyzygyPath) {
        throw "-Adjudicate and -SyzygyPath are contradictory; pick one label contract."
    }
    if ($SyzygyPath -and -not (Test-Path -LiteralPath $SyzygyPath -PathType Container)) {
        throw "-SyzygyPath is not a directory: $SyzygyPath"
    }
    $profile = if ($Adjudicate) {
        Get-DatagenProfile
    } elseif ($SyzygyPath) {
        Get-DatagenProfileV3 -SyzygyPath ((Resolve-Path $SyzygyPath).Path) -Pieces $SyzygyPieces
    } else {
        Get-DatagenProfileV2
    }
    # Hash before launch so the manifest identifies the inputs fastchess
    # actually opened, even if a file is changed after the run begins.
    $engineHash = Get-HarnessSha256 -Path $enginePath
    $binaryHashProperty = $engineManifest.PSObject.Properties["binary_sha256"]
    if (-not $binaryHashProperty -or -not $binaryHashProperty.Value) {
        throw "Datagen requires a hash-bound engine manifest; rebuild with tools\build_test.ps1."
    }
    if ($binaryHashProperty.Value -ne $engineHash) {
        throw "Engine binary SHA-256 does not match its sidecar; rebuild before generating labels."
    }
    foreach ($requiredOption in @("Hash", "Threads")) {
        if (-not (Test-EngineSupportsOption -Path $enginePath -Name $requiredOption)) {
            throw "Datagen engine does not advertise required UCI option '$requiredOption'."
        }
    }
    $bookHash = Get-HarnessSha256 -Path $Book
    $fastchessHash = Get-HarnessSha256 -Path $FastchessPath
    # Adjudication off by default since 2026-09-01 (RAR-M17). See
    # Get-DatagenProfileV2 for why the label-quality case is stronger here than
    # for a strength gate: the harm is sample depletion, not mislabeling.
    $adjudicationArgs = if ($Adjudicate) {
        @('-draw', "movenumber=$($profile.DrawMoveNumber)",
          "movecount=$($profile.DrawMoveCount)", "score=$($profile.DrawScore)") +
        @(Get-DatagenResignArgs)
    } elseif ($SyzygyPath) {
        @('-tb', $profile.TablebasePath,
          '-tbpieces', "$($profile.TablebasePieces)",
          '-tbadjudicate', 'BOTH')
    } else {
        @()
    }
    $fastchessArgs = @(
        '-engine', "cmd=$enginePath", 'name=A', "option.Hash=$Hash", 'option.Threads=1',
        '-engine', "cmd=$enginePath", 'name=B', "option.Hash=$Hash", 'option.Threads=1',
        '-each', 'tc=inf', "nodes=$Nodes",
        '-openings', "file=$Book", "format=$BookFormat", 'order=random', "start=$Start",
        '-srand', "$Seed",
        '-rounds', "$Rounds", '-games', '1',
        '-concurrency', "$Concurrency"
    ) + $adjudicationArgs + @(
        '-maxmoves', '200',
        '-pgnout', "file=$OutputPgn", 'append=false',
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
    $profileDetail = if ($Adjudicate) {
        "resign $($profile.ResignScore)/$($profile.ResignMoveCount), two-sided"
    } elseif ($SyzygyPath) {
        "Syzygy truth at $($profile.TablebasePieces) men, fifty-move rule kept"
    } else {
        "no adjudication; games play to a rules result"
    }
    Write-Host "  Profile : $($profile.Name) ($profileDetail)"
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

    $startedUtc = (Get-Date).ToUniversalTime()
    & $FastchessPath @fastchessArgs

    if ($LASTEXITCODE -ne 0) {
        throw "fastchess exited with code $LASTEXITCODE; partial PGN retained at $OutputPgn."
    }
    if (-not (Test-Path -LiteralPath $OutputPgn -PathType Leaf)) {
        throw "fastchess exited successfully without producing $OutputPgn."
    }

    Write-Host ""
    Write-Host "Done. PGN: $OutputPgn"

    $runManifest = [ordered]@{
        schema             = "rarog-fastchess-datagen-v2"
        started_utc        = $startedUtc.ToString("yyyy-MM-ddTHH:mm:ssZ")
        completed_utc      = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
        engine             = [ordered]@{
            path        = $enginePath
            sha256      = $engineHash
            manifest    = $engineManifestPath
            git_sha     = $engineManifest.git_sha
            git_tree    = $engineManifest.git_tree
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
        effective_threads  = 1
        concurrency        = $Concurrency
        fastchess          = [ordered]@{
            version = $fastchessInfo.Text
            sha256  = $fastchessHash
        }
        adjudication       = $profile
        max_moves          = 200
        output             = [ordered]@{
            path   = $OutputPgn
            bytes  = (Get-Item -LiteralPath $OutputPgn).Length
            sha256 = Get-HarnessSha256 -Path $OutputPgn
        }
    }
    Write-JsonAtomic -Path $outputManifest -Value $runManifest
    Write-Host "Manifest: $outputManifest"

    # Do not re-read a multi-GB PGN merely to count lines. The bounded preflight
    # below measures the quantity that matters: unique quiet yield per phase.
    Write-Host "Run extract.py --preflight-games 20000 on the 20k pilot before generating its continuation."
    Write-Host "The preflight sizes the exact total from measured limiting-phase unique yield."

} finally {
    Pop-Location
}
