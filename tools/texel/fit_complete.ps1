[CmdletBinding()]
param(
    [string[]]$Pgn = @(
        "tools/texel/data/selfplay-p1025a-zero-n8000-s1-g20000.pgn",
        "tools/texel/data/selfplay-p1025a-zero-n8000-s20001-g580000.pgn"
    ),
    [string]$DatasetDir = "tools/texel/data/hce-v2",
    [int]$TargetTrain = 2300000,
    [int]$Jobs = 14,
    [int]$NonlinearPositions = 200000,
    [int]$NonlinearEpochs = 40,
    [int]$LinearEpochs = 200,
    [int]$PolishEpochs = 60,
    [double]$LinearLearningRate = 0.3,
    [double]$LinearL2 = 0.0000001,
    [switch]$Smoke
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$invariant = [Globalization.CultureInfo]::InvariantCulture

function Format-Double {
    param([double]$Value)
    return $Value.ToString("R", $invariant)
}

function Parse-Double {
    param([string]$Value)
    return [double]::Parse($Value, [Globalization.NumberStyles]::Float, $invariant)
}

$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot "../.."))
Set-Location -LiteralPath $repo
$gitRoot = (& git rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0 -or [IO.Path]::GetFullPath($gitRoot) -ne $repo) {
    throw "fit_complete.ps1 must run inside the Rarog repository"
}
$trackedStatus = (& git status --short --untracked-files=no)
if ($LASTEXITCODE -ne 0) { throw "git status failed" }
if ($trackedStatus -and -not $Smoke) {
    throw "tracked worktree changes present; commit or restore them before fitting"
}
if ($trackedStatus -and $Smoke) {
    Write-Warning "SMOKE mode is running against tracked tooling changes; production mode refuses this"
}

$stamp = Get-Date -Format "yyyyMMdd_HHmmss"
$runDir = Join-Path $repo "tools/results/hce-fit-$stamp"
New-Item -ItemType Directory -Path $runDir | Out-Null

function Invoke-Logged {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$File,
        [Parameter(Mandatory)][string[]]$Arguments
    )
    $log = Join-Path $runDir "$Name.log"
    Write-Host "`n== $Name =="
    & $File @Arguments *> $log
    $exitCode = $LASTEXITCODE
    Get-Content -LiteralPath $log | ForEach-Object { Write-Host $_ }
    if ($exitCode -ne 0) {
        throw "$Name failed with exit code $exitCode; see $log"
    }
    return $log
}

function Invoke-Bench {
    param([Parameter(Mandatory)][string]$Name, [Parameter(Mandatory)][string]$Engine)
    $inputPath = Join-Path $runDir "$Name.input.txt"
    $stdoutPath = Join-Path $runDir "$Name.stdout.log"
    $stderrPath = Join-Path $runDir "$Name.stderr.log"
    Set-Content -LiteralPath $inputPath -Value "bench 13" -Encoding ascii
    $process = Start-Process -FilePath $Engine -WindowStyle Hidden -Wait -PassThru `
        -RedirectStandardInput $inputPath -RedirectStandardOutput $stdoutPath `
        -RedirectStandardError $stderrPath
    if ($process.ExitCode -ne 0) {
        throw "$Name failed with exit code $($process.ExitCode)"
    }
    $text = (Get-Content -LiteralPath $stdoutPath -Raw) + (Get-Content -LiteralPath $stderrPath -Raw)
    Set-Content -LiteralPath (Join-Path $runDir "$Name.log") -Value $text -Encoding utf8
    Write-Host $text
    $nodesMatch = [regex]::Match($text, "Nodes searched\s*:\s*([0-9]+)")
    $ebfMatch = [regex]::Match($text, "Geomean EBF\s*:\s*([0-9.]+)")
    if (-not $nodesMatch.Success -or -not $ebfMatch.Success) {
        throw "$Name did not emit the benchmark fingerprint"
    }
    return [pscustomobject]@{
        Nodes = [int64]$nodesMatch.Groups[1].Value
        Ebf = Parse-Double $ebfMatch.Groups[1].Value
        Log = Join-Path $runDir "$Name.log"
    }
}

# The accepted head's `bench 13` fingerprint. Bumped again 2026-09-02 when
# RAR-E08 accepted the tablebase-corrected label fit: 7,226,051 / 2.460 became
# 7,165,683 / 2.462. Bumped 2026-09-01 when RAR-E06's
# complete HCE refit was accepted: 6,977,070 / 2.466 (RAR-S70) became
# 7,226,051 / 2.460. It was NOT bumped at acceptance and the first fit run
# afterwards failed here -- correctly, but for a stale reason.
#
# WHAT THIS CANNOT PROVE. A fingerprint identifies the SEARCH, and a change
# confined to positions the bench suite never reaches is invisible to it. The
# 4.9a.4 mate drive is exactly that: it moves KBN-K conversion from 19.4% to
# 96.9% and leaves `bench 13` byte-identical, because no bench tree reaches a
# bare-king minor-piece mate. So this guard will happily pass a tree carrying
# an unaccepted eval change. Check `git rev-parse HEAD` against the commit the
# fit is supposed to start from; the run manifest records it for that purpose.
$script:AcceptedBenchNodes = 6901489
$script:AcceptedBenchEbf = 2.458

function Assert-BaselineFingerprint {
    param($Bench, [string]$Label)
    if ($Bench.Nodes -ne $script:AcceptedBenchNodes -or
        [Math]::Abs($Bench.Ebf - $script:AcceptedBenchEbf) -gt 0.0005) {
        throw ("$Label fingerprint is $($Bench.Nodes) / $($Bench.Ebf), expected " +
               "$script:AcceptedBenchNodes / $script:AcceptedBenchEbf")
    }
}

function Resolve-RepoPath {
    param([string]$Path)
    $full = [IO.Path]::GetFullPath((Join-Path $repo $Path))
    if (-not $full.StartsWith($repo + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw "path escapes repository: $Path"
    }
    return $full
}

function Get-WdlCsvAudit {
    param([Parameter(Mandatory)][string]$Path)
    $counts = @{ "0" = 0L; "0.5" = 0L; "1" = 0L }
    $rows = 0L
    $reader = [IO.File]::OpenText($Path)
    try {
        while ($null -ne ($line = $reader.ReadLine())) {
            $sep = $line.LastIndexOf(';')
            if ($sep -lt 0) { throw "$Path row $($rows + 1) has no target separator" }
            $target = $line.Substring($sep + 1).Trim()
            if (-not $counts.ContainsKey($target)) {
                throw "$Path row $($rows + 1) has non-WDL target '$target'"
            }
            $counts[$target]++
            $rows++
        }
    } finally {
        $reader.Dispose()
    }
    return [pscustomobject]@{ Rows = $rows; Counts = $counts }
}

function Get-EpdOpeningAudit {
    param([Parameter(Mandatory)][string]$Path)
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $rows = 0L
    foreach ($line in [IO.File]::ReadLines($Path)) {
        $parts = $line.Split(' ', [StringSplitOptions]::RemoveEmptyEntries)
        if ($parts.Count -lt 4) { throw "$Path opening $($rows + 1) is not a four-field FEN" }
        $fen4 = $parts[0..3] -join ' '
        if (-not $seen.Add($fen4)) { throw "$Path repeats opening '$fen4'" }
        $rows++
    }
    return $rows
}

$sourcePath = Join-Path $repo "src/eval.rs"
$sourceBackup = Join-Path $runDir "eval.rs.baseline"
Copy-Item -LiteralPath $sourcePath -Destination $sourceBackup
$sourceHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $sourcePath).Hash
$sourceRestored = $false

try {
    $commit = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) { throw "git rev-parse HEAD failed" }

    if ($Smoke) {
        $train = Resolve-RepoPath "tools/texel/data/train.csv"
        $validation = Resolve-RepoPath "tools/texel/data/holdout.csv"
        $test = $validation
        $NonlinearPositions = 20
        $NonlinearEpochs = 1
        $LinearEpochs = 1
        $PolishEpochs = 1
        $linearMax = 1000
        $datasetManifest = $null
        $testMarker = $null
        Write-Host "SMOKE mode: legacy data, tiny bounded fits, no dataset publication"
    } else {
        $dataset = Resolve-RepoPath $DatasetDir
        $manifestPath = Join-Path $dataset "manifest.json"
        if (-not (Test-Path -LiteralPath $manifestPath)) {
            if (Test-Path -LiteralPath $dataset) {
                throw "$dataset exists without manifest.json; refusing ambiguous publication target"
            }
            $pgnPaths = @($Pgn | ForEach-Object {
                $resolved = Resolve-RepoPath $_
                if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) { throw "missing PGN $resolved" }
                $resolved
            })
            $extractArgs = @(
                "tools/texel/extract_parallel.py"
            ) + $pgnPaths + @(
                "--out-dir", $dataset,
                "--target-train", [string]$TargetTrain,
                "--jobs", [string]$Jobs
            )
            [void](Invoke-Logged "dataset-audit" "python" ($extractArgs + "--audit-only"))
            [void](Invoke-Logged "dataset-publish" "python" $extractArgs)
        }
        # Validate both reused and freshly published corpora through the same
        # path. Publication success is not a substitute for checking what was
        # actually written.
        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        $expectedHeldout = [Math]::Round($TargetTrain / 18.0)
        # A NAMED WHITELIST, not a relaxation. The point of this check is that a
        # corpus whose labels mean something different must not be fitted as if
        # they meant self-play WDL -- and it correctly refused RAR-E08's arm B,
        # whose <=6-man positions carry Syzygy verdicts instead of game results.
        # The fix is to name the second contract, not to widen the test: an
        # unrecognised label string still throws, and each accepted contract is
        # written down here with what it means.
        $acceptedLabels = @(
            # hce-v2 and any corpus generated the same way.
            "white-perspective self-play WDL",
            # RAR-E08 arm B: identical rows, with every position of 6 men or
            # fewer relabelled to its tablebase verdict, cursed wins counted as
            # draws. See tools/texel/relabel_tb.py.
            "white-perspective self-play WDL, <=6-man Syzygy corrected"
        )
        if ($manifest.schema -ne "rarog-hce-wdl-v2" -or
            $acceptedLabels -notcontains $manifest.label -or
            [double]$manifest.train_blend -ne 1.0) {
            throw ("dataset label contract '$($manifest.label)' is not one this " +
                   "runner knows how to fit; accepted: $($acceptedLabels -join ' | ')")
        }
        if ([int]$manifest.rows.train -ne $TargetTrain) {
            throw "dataset train rows are $($manifest.rows.train), expected $TargetTrain"
        }
        if ([int]$manifest.rows.validation -ne $expectedHeldout -or
            [int]$manifest.rows.test -ne $expectedHeldout) {
            throw "dataset validation/test sizes do not match the frozen 5%/5% contract"
        }
        if ([int]$manifest.parse_errors -ne 0 -or
            [int]$manifest.paired_replays_discarded -ne 0) {
            throw "dataset records parse errors or replayed starts"
        }
        # A NAMED LIST of corpus contracts, exactly as $acceptedLabels above is
        # a named list of label contracts, and for the same reason: the check
        # exists so a corpus generated a different way is not fitted as if it
        # were the one the accepted head came from. Adding a corpus means
        # NAMING it here with the profile and start count it was actually built
        # at -- never widening the comparison or deleting it. An unlisted
        # profile, or a listed one at the wrong start count, still throws.
        #
        # The start count is pinned per contract because it is the evidence
        # that every opening was used exactly once: the extractor counts it,
        # the datagen provenance below must sum to it, and the book audit
        # proves the ranges never wrapped. A count that drifts means a segment
        # was rerun or a book was reused.
        $acceptedCorpora = @(
            # hce-v2: 750,000-opening phase-BALANCED beast_seed.epd, adjudicated
            # self-play. The corpus RAR-E06 and RAR-E08 were fitted on.
            [pscustomobject]@{ Adjudication = "datagen-v1"; Starts = 600000 },
            # hce-v3 (4.9a.6): 1,000,000-opening phase-WEIGHTED phase_book_v1.epd
            # at 50/10/10/10/20, NO adjudication. The balanced book could not
            # reach the row target at any schedule; see
            # analysis/texel_corpus_book_shape_2026-09-02.md.
            [pscustomobject]@{ Adjudication = "datagen-v2"; Starts = 602619 }
        )
        $corpusProfile = [string]$manifest.label_contract.adjudication.Name
        $corpusStarts = [int]$manifest.independent_starts
        $corpusMatch = @($acceptedCorpora | Where-Object {
            $_.Adjudication -eq $corpusProfile -and $_.Starts -eq $corpusStarts
        })
        if ($corpusMatch.Count -ne 1) {
            $known = ($acceptedCorpora | ForEach-Object {
                "$($_.Adjudication)/$($_.Starts)"
            }) -join ' | '
            throw ("corpus contract '$corpusProfile' at $corpusStarts independent " +
                   "starts is not one this runner knows how to fit; accepted: $known")
        }
        if ([int]$manifest.recorded_games -ne $corpusStarts) {
            throw ("recorded games $([int]$manifest.recorded_games) do not equal " +
                   "$corpusStarts independent starts; a start was replayed")
        }
        $bookOpenings = $null
        $bookAudited = $false
        $usedOpenings = 0
        foreach ($input in $manifest.inputs) {
            $provenancePath = [string]$input.provenance_manifest
            if (-not (Test-Path -LiteralPath $provenancePath -PathType Leaf)) {
                throw "missing datagen provenance $provenancePath"
            }
            $actualProvenanceHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $provenancePath).Hash
            if ($actualProvenanceHash -ne [string]$input.provenance_manifest_sha256) {
                throw "datagen provenance hash mismatch for $provenancePath"
            }
            $provenance = Get-Content -LiteralPath $provenancePath -Raw | ConvertFrom-Json
            if ($null -eq $bookOpenings) { $bookOpenings = [int]$provenance.book.openings }
            if ([int]$provenance.book.openings -ne $bookOpenings -or
                [string]$provenance.book.sha256 -ne [string]$manifest.label_contract.book_sha256 -or
                [int]$provenance.book.seed -ne [int]$manifest.label_contract.book_seed) {
                throw "datagen inputs do not share one opening-book contract"
            }
            if (-not $bookAudited) {
                if ([string]$provenance.book.format -ne "epd") {
                    throw "qualified HCE datagen requires an auditable EPD opening book"
                }
                $bookPath = [string]$provenance.book.path
                if (-not (Test-Path -LiteralPath $bookPath -PathType Leaf)) {
                    throw "missing opening book $bookPath"
                }
                if ((Get-FileHash -Algorithm SHA256 -LiteralPath $bookPath).Hash -ne
                    [string]$provenance.book.sha256) {
                    throw "opening-book hash mismatch for $bookPath"
                }
                $uniqueBookRows = Get-EpdOpeningAudit $bookPath
                if ($uniqueBookRows -ne $bookOpenings) {
                    throw "opening book contains $uniqueBookRows unique starts, manifest records $bookOpenings"
                }
                $bookAudited = $true
            }
            if ([int]$provenance.book.end -gt $bookOpenings) {
                throw "datagen range wraps the $bookOpenings-opening book"
            }
            $usedOpenings += [int]$provenance.games
        }
        if ($bookOpenings -lt [int]$manifest.independent_starts -or
            $usedOpenings -ne [int]$manifest.independent_starts) {
            throw "opening book is too small or its recorded ranges do not cover every independent start once"
        }
        foreach ($split in @("train", "validation", "test")) {
            $file = Join-Path $dataset "$split.csv"
            if (-not (Test-Path -LiteralPath $file)) { throw "missing frozen dataset file $file" }
            $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $file).Hash
            $expected = $manifest.output_sha256.$split
            if ($actual -ne $expected) { throw "$split.csv hash mismatch" }
            $wdl = Get-WdlCsvAudit $file
            if ($wdl.Rows -ne [int]$manifest.rows.$split) {
                throw "$split.csv contains $($wdl.Rows) rows, expected $($manifest.rows.$split)"
            }
            Write-Host ("  {0}: {1:N0} pure-WDL rows (0={2:N0}, 0.5={3:N0}, 1={4:N0})" -f `
                $split, $wdl.Rows, $wdl.Counts["0"], $wdl.Counts["0.5"], $wdl.Counts["1"])
        }
        Write-Host "Hash/row/WDL-verified frozen dataset $dataset"
        Write-Host "Opening provenance: $usedOpenings independent starts from a $bookOpenings-opening book"
        $train = Join-Path $dataset "train.csv"
        $validation = Join-Path $dataset "validation.csv"
        $test = Join-Path $dataset "test.csv"
        $datasetManifest = Join-Path $dataset "manifest.json"
        $testMarker = Join-Path $dataset "frozen-test.opened"
        if (Test-Path -LiteralPath $testMarker) {
            throw "frozen test was already consumed; see $testMarker"
        }
        $linearMax = 0
    }

    $settings = [ordered]@{
        schema = "rarog-complete-hce-fit-v2"
        commit = $commit
        source_sha256 = $sourceHash
        smoke = [bool]$Smoke
        target_train = $TargetTrain
        nonlinear_positions = $NonlinearPositions
        nonlinear_epochs = $NonlinearEpochs
        linear_epochs = $LinearEpochs
        polish_epochs = $PolishEpochs
        linear_learning_rate = $LinearLearningRate
        linear_l2 = $LinearL2
        train = $train
        validation = $validation
        frozen_test = $test
        frozen_test_baseline = (Join-Path $runDir "00-source-defaults.txt")
        label_contract = "pure white-perspective self-play WDL"
        schedule = @("nonlinear", "complete-linear", "nonlinear", "complete-linear-polish")
    }
    $settings | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $runDir "settings.json") -Encoding utf8

    [void](Invoke-Logged "build-tuner" "cargo" @("build", "--release", "-p", "texel-tuner"))
    $tuner = Join-Path $repo "target/release/rarog-texel.exe"
    $baselineVector = Join-Path $runDir "00-source-defaults.txt"
    [void](Invoke-Logged "write-source-defaults" $tuner @("--write-defaults", $baselineVector))
    [void](Invoke-Logged "instrument-coverage" $tuner @("--audit-coverage"))
    [void](Invoke-Logged "trace-verify-baseline" $tuner @("--verify", $validation))
    $supportArgs = @("--feature-support", $train)
    if ($linearMax -gt 0) { $supportArgs += @("--max-positions", [string]$linearMax) }
    [void](Invoke-Logged "feature-support" $tuner $supportArgs)
    $bucketLog = Invoke-Logged "baseline-buckets" $tuner @("--buckets", $validation)
    $bucketText = Get-Content -LiteralPath $bucketLog -Raw
    $kMatch = [regex]::Match($bucketText, "K = ([0-9]+(?:\.[0-9]+)?)")
    if (-not $kMatch.Success) { throw "could not parse fitted K" }
    $fixedK = $kMatch.Groups[1].Value
    Write-Host "Pinned K for every stage: $fixedK"

    [void](Invoke-Logged "build-baseline" "cargo" @("build", "--release", "-p", "rarog", "--bin", "rarog"))
    $engine = Join-Path $repo "target/release/rarog.exe"
    $baselineBench = Invoke-Bench "bench-baseline" $engine
    Assert-BaselineFingerprint $baselineBench "initial baseline"
    Copy-Item -LiteralPath $engine -Destination (Join-Path $runDir "rarog-baseline.exe")

    $ks1 = Join-Path $runDir "01-kingsafety.txt"
    $linear1 = Join-Path $runDir "02-complete-linear.txt"
    $ks2 = Join-Path $runDir "03-kingsafety.txt"
    $final = Join-Path $runDir "04-final.txt"

    [void](Invoke-Logged "fit-01-kingsafety" $tuner @(
        "--tune-kingsafety", $train, $validation, $ks1,
        "--epochs", [string]$NonlinearEpochs,
        "--max-positions", [string]$NonlinearPositions,
        "--fix-k", $fixedK
    ))

    $linearArgs = @(
        "--tune", "complete", $train, $validation, $linear1,
        "--initial", $ks1,
        "--epochs", [string]$LinearEpochs,
        "--lr", (Format-Double $LinearLearningRate),
        "--l2", (Format-Double $LinearL2),
        "--fix-k", $fixedK
    )
    if ($linearMax -gt 0) { $linearArgs += @("--max-positions", [string]$linearMax) }
    [void](Invoke-Logged "fit-02-complete-linear" $tuner $linearArgs)

    [void](Invoke-Logged "fit-03-kingsafety" $tuner @(
        "--tune-kingsafety", $train, $validation, $ks2,
        "--initial", $linear1,
        "--epochs", [string]$NonlinearEpochs,
        "--max-positions", [string]$NonlinearPositions,
        "--fix-k", $fixedK
    ))

    $polishArgs = @(
        "--tune", "complete", $train, $validation, $final,
        "--initial", $ks2,
        "--test", $test,
        "--test-baseline", $baselineVector,
        "--epochs", [string]$PolishEpochs,
        "--lr", (Format-Double $LinearLearningRate),
        "--l2", (Format-Double $LinearL2),
        "--fix-k", $fixedK
    )
    if ($linearMax -gt 0) { $polishArgs += @("--max-positions", [string]$linearMax) }
    if ($testMarker) { $polishArgs += @("--test-marker", $testMarker) }
    $finalFitLog = Invoke-Logged "fit-04-final-polish" $tuner $polishArgs
    [void](Invoke-Logged "trace-verify-final-vector" $tuner @("--verify", $validation, "--weights", $final))

    [void](Invoke-Logged "bake-final-vector" "python" @("tools/texel/bake_params.py", $final))
    $candidateHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $sourcePath).Hash
    if ($candidateHash -eq $sourceHash) { throw "final vector baked no source movement" }
    [void](Invoke-Logged "format-candidate" "cargo" @("fmt"))
    [void](Invoke-Logged "format-check-candidate" "cargo" @("fmt", "--check"))
    if (-not $Smoke) {
        [void](Invoke-Logged "test-candidate-debug" "cargo" @("test", "--workspace", "--all-targets"))
        [void](Invoke-Logged "test-candidate-release" "cargo" @("test", "--workspace", "--all-targets", "--release"))
        [void](Invoke-Logged "clippy-candidate" "cargo" @("clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"))
    }
    [void](Invoke-Logged "build-candidate" "cargo" @("build", "--release", "-p", "rarog", "--bin", "rarog"))
    $candidateBench = Invoke-Bench "bench-candidate" $engine
    Copy-Item -LiteralPath $engine -Destination (Join-Path $runDir "rarog-candidate.exe")
    $candidatePatch = Join-Path $runDir "candidate-eval.patch"
    $candidatePatchStderr = Join-Path $runDir "candidate-eval.patch.stderr.log"
    & git diff "--output=$candidatePatch" -- src/eval.rs 2> $candidatePatchStderr
    if ($LASTEXITCODE -ne 0) { throw "git diff failed" }
    if (-not (Test-Path -LiteralPath $candidatePatch) -or
        (Get-Item -LiteralPath $candidatePatch).Length -eq 0) {
        throw "candidate patch is empty"
    }

    Copy-Item -LiteralPath $sourceBackup -Destination $sourcePath -Force
    [void](Invoke-Logged "format-restored" "cargo" @("fmt"))
    $restoredHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $sourcePath).Hash
    if ($restoredHash -ne $sourceHash) { throw "src/eval.rs did not restore byte-for-byte" }
    [void](Invoke-Logged "check-candidate-patch" "git" @("apply", "--check", $candidatePatch))
    # The restored file can have an older timestamp than the freshly built
    # candidate. A plain cargo build then legally reuses the candidate binary.
    # Advance only its mtime (the content hash above remains the proof) so Cargo
    # must rebuild it. `cargo clean -p rarog` does not reliably remove the final
    # executable/fingerprint on this workspace layout.
    [IO.File]::SetLastWriteTimeUtc($sourcePath, [DateTime]::UtcNow)
    [void](Invoke-Logged "build-restored-baseline" "cargo" @("build", "--release", "-p", "rarog", "--bin", "rarog"))
    $restoredBench = Invoke-Bench "bench-restored-baseline" $engine
    Assert-BaselineFingerprint $restoredBench "restored baseline"
    $sourceRestored = $true

    $finalFitText = Get-Content -LiteralPath $finalFitLog -Raw
    $validationMatch = [regex]::Match(
        $finalFitText,
        "Best validation epoch ([0-9]+) \(validation=([0-9.]+)\)"
    )
    $persistedValidationMatch = [regex]::Match(
        $finalFitText,
        "Persisted rounded validation loss = ([0-9.]+)"
    )
    $testMatch = [regex]::Match(
        $finalFitText,
        "Frozen test loss = ([0-9.]+) \(source baseline ([0-9.]+), delta ([+-][0-9.]+)\)"
    )
    if (-not $validationMatch.Success -or -not $persistedValidationMatch.Success -or
        -not $testMatch.Success) {
        throw "could not parse final validation/test results from $finalFitLog"
    }

    $summary = [ordered]@{
        schema = "rarog-complete-hce-fit-result-v2"
        status = "complete"
        run_dir = $runDir
        final_vector = $final
        fixed_k = Parse-Double $fixedK
        baseline = @{ nodes = $baselineBench.Nodes; ebf = $baselineBench.Ebf }
        candidate = @{ nodes = $candidateBench.Nodes; ebf = $candidateBench.Ebf }
        restored_baseline = @{ nodes = $restoredBench.Nodes; ebf = $restoredBench.Ebf }
        final_validation = @{
            best_epoch = [int]$validationMatch.Groups[1].Value
            selection_loss = Parse-Double $validationMatch.Groups[2].Value
            persisted_rounded_loss = Parse-Double $persistedValidationMatch.Groups[1].Value
        }
        frozen_test = @{
            loss = Parse-Double $testMatch.Groups[1].Value
            source_baseline_loss = Parse-Double $testMatch.Groups[2].Value
            delta = Parse-Double $testMatch.Groups[3].Value
        }
        dataset_manifest_sha256 = if ($datasetManifest) {
            (Get-FileHash -Algorithm SHA256 -LiteralPath $datasetManifest).Hash
        } else { $null }
        frozen_test_marker = if ($testMarker) { $testMarker } else { $null }
        frozen_test_marker_sha256 = if ($testMarker) {
            (Get-FileHash -Algorithm SHA256 -LiteralPath $testMarker).Hash
        } else { $null }
        final_vector_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $final).Hash
        source_vector_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $baselineVector).Hash
        candidate_patch_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $candidatePatch).Hash
        candidate_exe_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $runDir "rarog-candidate.exe")).Hash
        source_restored = $true
        frozen_test_opened_once = [bool]$testMarker
        strength_verdict = "not run; register SPRT only after reviewing this offline fit"
    }
    $summary | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $runDir "summary.json") -Encoding utf8
    Write-Host "`nCOMPLETE: $runDir"
    Write-Host "Final vector: $final"
    Write-Host "Frozen test: $($testMatch.Groups[1].Value) (delta $($testMatch.Groups[3].Value))"
    Write-Host "Candidate bench: $($candidateBench.Nodes) / $($candidateBench.Ebf)"
    Write-Host "Source restored and release binary rebuilt to $script:AcceptedBenchNodes / $script:AcceptedBenchEbf."
} finally {
    if (-not $sourceRestored) {
        Copy-Item -LiteralPath $sourceBackup -Destination $sourcePath -Force
        $restoredHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $sourcePath).Hash
        if ($restoredHash -ne $sourceHash) {
            Write-Error "EMERGENCY RESTORE FAILED for src/eval.rs"
        } else {
            Write-Warning "Restored src/eval.rs after interrupted/failed run; rebuilding the normal release binary"
            [IO.File]::SetLastWriteTimeUtc($sourcePath, [DateTime]::UtcNow)
            & cargo build --release -p rarog --bin rarog
            $rebuildExit = $LASTEXITCODE
            if ($rebuildExit -ne 0) {
                Write-Error "Emergency normal release rebuild failed with exit code $rebuildExit; do not measure target/release/rarog.exe"
            } else {
                Write-Host "Normal release binary rebuilt after emergency restore."
            }
        }
    }
}
