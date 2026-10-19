<#
.SYNOPSIS
    Generate a deterministic self-play PGN segment for Texel tuning.

.DESCRIPTION
    Runs self-play between two copies of the given engine at a fixed node
    limit, one game per opening, collecting a large PGN file suitable for
    tools\texel\extract.py. Colosseum plays the games by default; fastchess
    stays available as the backup runner (-Runner fastchess).

    A fixed -Seed shuffles the opening book reproducibly in fastchess's order.
    -Start and -Rounds select a non-wrapping range in that shuffled order, so a
    pilot and its continuation cannot silently replay openings. The output
    filename records the engine suffix, node count, start, and game count.

    Colosseum shuffles with its own generator, so it does not shuffle here: it
    plays -OpeningSequence, the book's lines already in fastchess's seeded order
    (written by tools\texel\opening_sequence.py from earlier fastchess corpora),
    in sequential order. A segment therefore means the same openings under
    either runner, and the corpora keep the same starts.

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
    intended to partition one book. Default 10403.

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

.PARAMETER Runner
    colosseum (default) or fastchess. Colosseum runs the no-adjudication
    contract only; -Adjudicate and -SyzygyPath need -Runner fastchess.

.PARAMETER OpeningSequence
    Colosseum only: the ordered EPD for -Book and -Seed. Default:
    <book directory>\<book name>-s<Seed>.order.epd. Its sidecar JSON must name
    the book's SHA-256, the seed and the sequence file's own SHA-256.

.PARAMETER ColosseumPath
    Path to colosseum-cli.exe. Default: tools\bin\colosseum-cli.exe, which must
    be the build tools\colosseum\colosseum.pin.json pins.

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
    [ValidateSet("colosseum", "fastchess")]
    [string]$Runner      = "colosseum",
    [string]$OpeningSequence = "",
    [string]$ColosseumPath = "",
    [switch]$Append,
    # Opt back into datagen-v1 adjudication. Off by default since 2026-09-01.
    [switch]$Adjudicate,
    # Syzygy WDL directory. When set, games are adjudicated on TABLEBASE TRUTH
    # at 6 men (profile datagen-v3) instead of being played out at the datagen
    # node budget, which the engine often cannot convert. Never use this for a
    # strength gate. Mutually exclusive with -Adjudicate.
    [string]$SyzygyPath = "",
    [int]$SyzygyPieces = 6,
    # Registered identity of the label generator: its SHA-256 and its
    # `bench 13` node count. When given, a binary that differs or benches
    # anything else is refused before the first game (the sidecar alone
    # only proves the binary matches its own record).
    [string]$ExpectSha256 = "",
    [long]$ExpectFingerprint = 0,
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
    if (-not $ColosseumPath) { $ColosseumPath = "$PSScriptRoot\bin\colosseum-cli.exe" }
    $enginePath = "$PSScriptRoot\test_engines\rarog-$Suffix-pext-pgo.exe"
    $runnerPath = if ($Runner -eq "fastchess") { $FastchessPath } else { $ColosseumPath }

    foreach ($p in @($Book, $runnerPath, $enginePath)) {
        if (-not (Test-Path $p)) { throw "Not found: $p" }
    }
    $enginePath   = (Resolve-Path $enginePath).Path
    $Book         = (Resolve-Path $Book).Path
    $runnerPath   = (Resolve-Path $runnerPath).Path

    if ($Runner -eq "colosseum") {
        if ($Adjudicate -or $SyzygyPath) {
            throw "-Adjudicate and -SyzygyPath are legacy label contracts; run them with -Runner fastchess."
        }
        if (-not $PSBoundParameters.ContainsKey('BookFormat') -and
            [IO.Path]::GetExtension($Book) -eq ".epd") {
            $BookFormat = "epd"
        }
        if ($BookFormat -ne "epd") {
            throw "-Runner colosseum plays an ordered EPD sequence; -Book must be an EPD book."
        }
        if (-not $OpeningSequence) {
            $OpeningSequence = Join-Path (Split-Path -Parent $Book) (
                [IO.Path]::GetFileNameWithoutExtension($Book) + "-s$Seed.order.epd")
        }
        if (-not (Test-Path -LiteralPath $OpeningSequence -PathType Leaf)) {
            throw ("Opening sequence not found: $OpeningSequence. Write it with " +
                   "tools\texel\opening_sequence.py from fastchess corpora of this book and seed.")
        }
        $OpeningSequence = (Resolve-Path -LiteralPath $OpeningSequence).Path
    }

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

    # Book-diversity guard: fixed-node self-play from a
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

    # The sequence Colosseum plays must be this book in this seed's order, and
    # a segment must lie inside the part of that order already recovered.
    $sequenceInfo = $null
    if ($Runner -eq "colosseum") {
        $sidecarPath = [IO.Path]::ChangeExtension($OpeningSequence, ".json")
        if (-not (Test-Path -LiteralPath $sidecarPath -PathType Leaf)) {
            throw "Opening sequence has no sidecar: $sidecarPath"
        }
        $sidecar = Get-Content -LiteralPath $sidecarPath -Raw | ConvertFrom-Json
        $sequenceHash = Get-HarnessSha256 -Path $OpeningSequence
        if ($sidecar.schema -ne "rarog-opening-sequence-v1") {
            throw "$sidecarPath is not a rarog-opening-sequence-v1 record."
        }
        if ($sidecar.book.sha256 -ne (Get-HarnessSha256 -Path $Book)) {
            throw "Opening sequence $OpeningSequence was written for another book ($($sidecar.book.sha256))."
        }
        if ([int]$sidecar.seed -ne $Seed) {
            throw "Opening sequence $OpeningSequence is seed $($sidecar.seed)'s order, not seed $Seed's."
        }
        if ($sidecar.output.sha256 -ne $sequenceHash) {
            throw "Opening sequence $OpeningSequence is not the file its sidecar records."
        }
        $sequenceOpenings = [int]$sidecar.openings
        $sequenceLines = 0
        foreach ($line in [IO.File]::ReadLines($OpeningSequence)) { $sequenceLines++ }
        if ($sequenceLines -ne $sequenceOpenings) {
            throw "Opening sequence holds $sequenceLines lines; its sidecar records $sequenceOpenings."
        }
        if ($Start -gt $sequenceOpenings) {
            throw "Start $Start lies past the $sequenceOpenings openings recovered in $OpeningSequence."
        }
        $remaining = $sequenceOpenings - $Start + 1
        $sequenceInfo = [ordered]@{
            path           = $OpeningSequence
            sha256         = $sequenceHash
            openings       = $sequenceOpenings
            sidecar        = $sidecarPath
            sidecar_sha256 = Get-HarnessSha256 -Path $sidecarPath
        }
    }
    if ($Rounds -eq 0) { $Rounds = $remaining }
    if ($Rounds -gt $remaining) {
        $limit = if ($sequenceInfo) { "the $($sequenceInfo.openings) openings recovered in the sequence" } else { "the $openings-opening book" }
        throw "Segment [$Start, $($Start + $Rounds - 1)] exceeds $limit and would wrap/reuse openings. Maximum Rounds from Start=$Start is $remaining."
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
    # Colosseum's durable run directory: an interrupted segment resumes from it
    # when the same command is run again.
    $runDir = [IO.Path]::ChangeExtension($OutputPgn, ".colosseum")

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

    $runnerInfo = if ($Runner -eq "fastchess") {
        [pscustomobject]@{
            Text   = (Get-FastchessVersion -Path $runnerPath).Text
            Sha256 = Get-HarnessSha256 -Path $runnerPath
        }
    } else {
        $cli = Assert-ColosseumCli -Path $runnerPath -PinPath "$PSScriptRoot\colosseum\colosseum.pin.json"
        [pscustomobject]@{ Text = $cli.Version; Sha256 = $cli.Sha256; Revision = $cli.Pin.revision }
    }
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
    if ($ExpectSha256 -and $engineHash -ne $ExpectSha256.ToUpperInvariant()) {
        throw "Engine SHA-256 $engineHash is not the registered $ExpectSha256."
    }
    $verifiedBenchNodes = $null
    if ($ExpectFingerprint -gt 0) {
        $benchOut = & $enginePath bench 13 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) { throw "Engine bench 13 exited $LASTEXITCODE." }
        $benchMatch = [regex]::Match($benchOut, 'Nodes searched\s*:\s*(\d+)')
        if (-not $benchMatch.Success) { throw "Engine bench 13 printed no node count." }
        $verifiedBenchNodes = [long]$benchMatch.Groups[1].Value
        if ($verifiedBenchNodes -ne $ExpectFingerprint) {
            throw "Engine benches $verifiedBenchNodes, not the registered $ExpectFingerprint."
        }
        Write-Host "Engine verified: SHA-256 $engineHash, bench 13 $verifiedBenchNodes nodes."
    }
    foreach ($requiredOption in @("Hash", "Threads")) {
        if (-not (Test-EngineSupportsOption -Path $enginePath -Name $requiredOption)) {
            throw "Datagen engine does not advertise required UCI option '$requiredOption'."
        }
    }
    $bookHash = Get-HarnessSha256 -Path $Book
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
    $runnerArgs = if ($Runner -eq "fastchess") {
        @(
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
    } else {
        # A tournament of one engine against itself, one game per encounter and
        # one opening per encounter, read from the ordered sequence (Colosseum's
        # book start is zero-based). A fixed match would play each opening twice.
        @(
            'tournament', 'run', '--format', 'round-robin',
            '--engine', $enginePath, '--engine', $enginePath, '--label', 'A', '--label', 'B',
            '--games-per-pair', '1', '--cycles', "$Rounds",
            '--nodes', "$Nodes", '--option', "Hash=$Hash", '--option', 'Threads=1',
            '--book', $OpeningSequence, '--book-order', 'sequential', '--book-start', "$($Start - 1)",
            '--seed', "$Seed", '--max-moves', '200', '--concurrency', "$Concurrency",
            '--engine-processes', 'per-slot',
            '--progress-every', '2000', '--progress-min-secs', '60',
            '--dir', $runDir
        )
    }

    $games = $Rounds
    $orderText = if ($Runner -eq "fastchess") { "shuffled with seed $Seed" } else { "seed $Seed's order, from $(Split-Path $OpeningSequence -Leaf)" }
    Write-Host ""
    Write-Host "============================================================"
    Write-Host "  Rarog Texel datagen — self-play"
    Write-Host "  Engine  : $enginePath"
    Write-Host "  Games   : $games (one independent opening each)"
    Write-Host "  Segment : $Start..$segmentEnd of $openings ($orderText)"
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
    Write-Host "  Runner  : $($runnerInfo.Text)"
    Write-Host "  Output  : $OutputPgn"
    if ($Runner -eq "colosseum") {
        Write-Host "  Run dir : $runDir$(if (Test-Path -LiteralPath $runDir) { ' (exists: the run resumes)' })"
    }
    Write-Host "============================================================"
    Write-Host ""

    # NOTE (2026-07-22): datagen deliberately has NO affinity and may use
    # oversubscribed concurrency. Games are NODE-limited, so scheduler
    # placement cannot change a move or label; only throughput is affected.
    $quotedArgs = $runnerArgs | ForEach-Object {
        if ($_ -match '[\s"]') { '"' + ($_ -replace '"', '\"') + '"' } else { $_ }
    }
    Write-Host "Command  : & `"$runnerPath`" $($quotedArgs -join ' ')"

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
    # Colosseum refuses to resume a completed run, so a segment whose games
    # finished before the wrapper did goes straight to verification.
    $existingRecord = Join-Path $runDir "run-record.json"
    if ($Runner -eq "colosseum" -and (Test-Path -LiteralPath $existingRecord) -and
        (Get-Content -LiteralPath $existingRecord -Raw | ConvertFrom-Json).status -eq "completed") {
        Write-Host "The run in $runDir is already complete; verifying it without playing."
        $runExit = 0
    } else {
        & $runnerPath @runnerArgs
        $runExit = $LASTEXITCODE
    }

    $runnerRecord = $null
    if ($Runner -eq "fastchess") {
        if ($runExit -ne 0) {
            throw "fastchess exited with code $runExit; partial PGN retained at $OutputPgn."
        }
        if (-not (Test-Path -LiteralPath $OutputPgn -PathType Leaf)) {
            throw "fastchess exited successfully without producing $OutputPgn."
        }
    } else {
        $exit = Resolve-ColosseumExit -Mode "tournament" -ExitCode $runExit
        if ($exit.Kind -ne "outcome") {
            throw ("colosseum-cli exited $runExit ($($exit.Kind)). The run directory $runDir keeps " +
                   "its durable games; run the same command again to resume.")
        }
        # The runner's own records, not its console: completion, every game
        # scored, no engine fault, and the openings and limits it resolved.
        $recordPath = Join-Path $runDir "run-record.json"
        $resolvedPath = Join-Path $runDir "resolved-config.json"
        $record = Get-Content -LiteralPath $recordPath -Raw | ConvertFrom-Json
        if ($record.status -ne "completed") {
            throw "The Colosseum run's status is '$($record.status)', not completed. See $runDir."
        }
        $scored = [int]$record.official_sample.scored_games
        if ($scored -ne $Rounds) {
            throw "The Colosseum run scored $scored games, not $Rounds. See $runDir."
        }
        $status = Get-ColosseumRunStatus -CliPath $runnerPath -Dir $runDir
        Assert-ColosseumRunFaults -Status $status -ScoredGames $scored -TimeLossRateCeiling 0 -Dir $runDir | Out-Null
        $resolved = Get-Content -LiteralPath $resolvedPath -Raw | ConvertFrom-Json
        $o = $resolved.openings
        $expected = @(
            @("openings.path", [IO.Path]::GetFullPath($o.path), $OpeningSequence),
            @("openings.order", "$($o.order)", "Sequential"),
            @("openings.wrap", "$($o.wrap)", "False"),
            @("openings.first_index", "$($o.first_index)", "$($Start - 1)"),
            @("openings.last_index", "$($o.last_index)", "$($segmentEnd - 1)"),
            @("openings.reused_openings", "$($o.reused_openings)", "0"),
            @("time_control nodes", "$($resolved.time_control.control.Nodes.nodes)", "$Nodes"),
            @("adjudication.draw", "$($resolved.adjudication.draw)", ""),
            @("adjudication.resign", "$($resolved.adjudication.resign)", ""),
            @("adjudication.max_moves", "$($resolved.adjudication.max_moves)", "200"),
            @("participants", "$(@($resolved.plan.participants).Count)", "2"),
            @("games_per_pair", "$($resolved.plan.design.games_per_pair)", "1"),
            @("engine_processes", "$($resolved.engine_processes)", "per-slot")
        )
        foreach ($entry in @($resolved.plan.participants)) {
            $launch = $entry.participant.launch
            $expected += , @("participant executable", [IO.Path]::GetFullPath($launch.executable), $enginePath)
            $expected += , @("participant Hash", "$($launch.options.Hash.value)", "$Hash")
            $expected += , @("participant Threads", "$($launch.options.Threads.value)", "1")
        }
        foreach ($check in $expected) {
            if ($check[1] -ne $check[2]) {
                throw "The Colosseum run resolved $($check[0]) = '$($check[1])', expected '$($check[2])'. See $resolvedPath."
            }
        }
        # One game per opening, exactly the segment: compare the PGN's start
        # positions with the sequence's lines Start..End.
        $want = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        $index = 0
        foreach ($line in [IO.File]::ReadLines($OpeningSequence)) {
            $index++
            if ($index -lt $Start) { continue }
            if ($index -gt $segmentEnd) { break }
            [void]$want.Add((($line.Split(' ', [StringSplitOptions]::RemoveEmptyEntries))[0..3] -join ' '))
        }
        $runPgn = Join-Path $runDir "games.pgn"
        $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        $pgnGames = 0
        foreach ($line in [IO.File]::ReadLines($runPgn)) {
            if (-not $line.StartsWith('[FEN "')) { continue }
            $pgnGames++
            $fen = $line.Substring(6, $line.LastIndexOf('"') - 6)
            $key = ($fen.Split(' ', [StringSplitOptions]::RemoveEmptyEntries))[0..3] -join ' '
            if (-not $seen.Add($key)) { throw "$runPgn plays opening '$key' twice." }
            if (-not $want.Contains($key)) { throw "$runPgn plays opening '$key', which is outside the segment." }
        }
        if ($pgnGames -ne $Rounds -or $seen.Count -ne $want.Count) {
            throw "$runPgn holds $pgnGames games on $($seen.Count) openings; the segment has $($want.Count)."
        }
        Move-Item -LiteralPath $runPgn -Destination $OutputPgn
        $runnerRecord = [ordered]@{
            version                = $runnerInfo.Text
            sha256                 = $runnerInfo.Sha256
            revision               = $runnerInfo.Revision
            run_directory          = $runDir
            run_record_sha256      = Get-HarnessSha256 -Path $recordPath
            resolved_config_sha256 = Get-HarnessSha256 -Path $resolvedPath
            pgn                    = "moved from the run directory to output.path"
        }
    }

    Write-Host ""
    Write-Host "Done. PGN: $OutputPgn"

    $bookRecord = [ordered]@{
        path      = $Book
        format    = $BookFormat
        sha256    = $bookHash
        openings  = $openings
        seed      = $Seed
        start     = $Start
        end       = $segmentEnd
    }
    if ($sequenceInfo) { $bookRecord.sequence = $sequenceInfo }
    $runManifest = [ordered]@{
        schema             = if ($Runner -eq "fastchess") { "rarog-fastchess-datagen-v2" } else { "rarog-colosseum-datagen-v1" }
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
            verified_bench_nodes = $verifiedBenchNodes
            built_utc   = $engineManifest.built_utc
        }
        book               = $bookRecord
        games              = $Rounds
        nodes_per_move     = $Nodes
        hash_mb            = $Hash
        effective_threads  = 1
        concurrency        = $Concurrency
    }
    if ($Runner -eq "fastchess") {
        $runManifest.fastchess = [ordered]@{
            version = $runnerInfo.Text
            sha256  = $runnerInfo.Sha256
        }
    } else {
        $runManifest.colosseum = $runnerRecord
    }
    $runManifest.adjudication = $profile
    $runManifest.max_moves = 200
    $runManifest.output = [ordered]@{
        path   = $OutputPgn
        bytes  = (Get-Item -LiteralPath $OutputPgn).Length
        sha256 = Get-HarnessSha256 -Path $OutputPgn
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
