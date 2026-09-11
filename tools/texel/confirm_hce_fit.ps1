[CmdletBinding()]
param(
    [string]$FitRun = "tools/results/hce-fit-20260831_095443",
    [int]$Games = 150000,
    [int]$Start = 600001,
    [int]$Nodes = 8000,
    [int]$Seed = 10403,
    [int]$Jobs = 14,
    [int]$Concurrency = 0,
    # The label contract this run must prove it used. The check below exists to
    # prove the corpus carries the DECLARED contract, not to freeze one profile
    # for all time -- pinning the literal string meant the assertion would fail
    # the moment the datagen default moved, which is the wrong failure: it would
    # report a provenance defect when provenance was fine. Declare it here and
    # the check still catches a corpus generated under anything else.
    [ValidateSet("datagen-v1", "datagen-v2")]
    [string]$ExpectedAdjudication = "datagen-v2",
    [switch]$ValidateOnly
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$invariant = [Globalization.CultureInfo]::InvariantCulture
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot "../.."))
. (Join-Path $repo "tools/harness_common.ps1")
Set-Location -LiteralPath $repo

function Resolve-InputPath {
    param([Parameter(Mandatory)][string]$Path)
    $full = if ([IO.Path]::IsPathRooted($Path)) {
        [IO.Path]::GetFullPath($Path)
    } else {
        [IO.Path]::GetFullPath((Join-Path $repo $Path))
    }
    if (-not (Test-Path -LiteralPath $full)) { throw "Not found: $full" }
    return $full
}

function Invoke-Checked {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$File,
        [Parameter(Mandatory)][string[]]$Arguments
    )
    Write-Host "`n== $Name =="
    & $File @Arguments
    $exitCode = $LASTEXITCODE
    if ($exitCode -ne 0) { throw "$Name failed with exit code $exitCode" }
}

function Get-WdlCsvAudit {
    param([Parameter(Mandatory)][string]$Path)
    $counts = @{ "0" = 0L; "0.5" = 0L; "1" = 0L }
    $rows = 0L
    $reader = [IO.File]::OpenText($Path)
    try {
        while ($null -ne ($line = $reader.ReadLine())) {
            $separator = $line.LastIndexOf(';')
            if ($separator -lt 0) { throw "$Path row $($rows + 1) has no target separator" }
            $target = $line.Substring($separator + 1).Trim()
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

if ($Games -lt 1 -or $Start -lt 1 -or $Nodes -lt 1 -or $Seed -lt 1 -or $Jobs -lt 1) {
    throw "Games, Start, Nodes, Seed and Jobs must be positive"
}
if ($Start -ne 600001 -or $Games -ne 150000 -or ($Start + $Games - 1) -ne 750000) {
    throw "Confirmation is frozen to the unused book tail: Start=600001, Games=150000"
}

$gitRoot = (& git rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0 -or [IO.Path]::GetFullPath($gitRoot) -ne $repo) {
    throw "confirm_hce_fit.ps1 must run inside the Rarog repository"
}
$trackedStatus = & git status --short --untracked-files=no
if ($LASTEXITCODE -ne 0) { throw "git status failed" }
if ($trackedStatus) { throw "tracked worktree changes present; commit or restore them before confirmation" }

$fitDir = Resolve-InputPath $FitRun
$fitSummaryPath = Join-Path $fitDir "summary.json"
$fitSettingsPath = Join-Path $fitDir "settings.json"
$fitSummary = Get-Content -LiteralPath $fitSummaryPath -Raw | ConvertFrom-Json
$fitSettings = Get-Content -LiteralPath $fitSettingsPath -Raw | ConvertFrom-Json
if ($fitSummary.status -ne "complete" -or -not $fitSummary.final_vector) {
    throw "$fitSummaryPath does not describe a completed fit"
}
$candidateVector = Resolve-InputPath ([string]$fitSummary.final_vector)
if ((Get-HarnessSha256 $candidateVector) -ne [string]$fitSummary.final_vector_sha256) {
    throw "final candidate vector hash differs from the completed fit summary"
}
$evalPath = Join-Path $repo "src/eval.rs"
if ((Get-HarnessSha256 $evalPath) -ne [string]$fitSettings.source_sha256) {
    throw "src/eval.rs no longer matches the fitted source baseline"
}
& git diff --quiet ([string]$fitSettings.commit) -- src
if ($LASTEXITCODE -ne 0) {
    throw "engine source changed since the fit; this confirmation no longer compares the registered candidate"
}

$fixedK = [double]$fitSummary.fixed_k
$fixedKText = $fixedK.ToString("R", $invariant)
if ($ValidateOnly) {
    Write-Host "Validation passed: fixed candidate and unchanged engine source."
    Write-Host "Planned data: $Games pure-WDL games, starts $Start..$($Start + $Games - 1), then 127,778 untouched test positions."
    return
}
$stamp = Get-Date -Format "yyyyMMdd_HHmmss"
$suffix = "hce-confirm-$stamp"
$runDir = Join-Path $repo "tools/results/$suffix"
$datasetDir = Join-Path $runDir "dataset"
$pgn = Join-Path $runDir "selfplay-$suffix-n$Nodes-s$Start-g$Games.pgn"
$sourceVector = Join-Path $runDir "source-vector.txt"
$testMarker = Join-Path $datasetDir "frozen-confirmation.opened"
$compareLog = Join-Path $runDir "exact-confirmation.log"
New-Item -ItemType Directory -Path $runDir | Out-Null
$transcript = Join-Path $runDir "run-transcript.log"
$transcribing = $false

try {
    Start-Transcript -LiteralPath $transcript | Out-Null
    $transcribing = $true
    Write-Host "Fresh confirmation: $Games independent games from unused starts $Start..$($Start + $Games - 1)"
    Write-Host "Labels: pure self-play WDL; Stockfish evaluations are not read"
    Write-Host "Output: $runDir"

    Invoke-Checked "build clean PGO datagen engine" "pwsh" @(
        "-NoProfile", "-File", "tools/build_test.ps1", "-Suffix", $suffix
    )
    $datagenArgs = @(
        "-NoProfile", "-File", "tools/datagen.ps1",
        "-Suffix", $suffix,
        "-Rounds", [string]$Games,
        "-Start", [string]$Start,
        "-Seed", [string]$Seed,
        "-Nodes", [string]$Nodes,
        "-OutputPgn", $pgn
    )
    if ($Concurrency -gt 0) { $datagenArgs += @("-Concurrency", [string]$Concurrency) }
    Invoke-Checked "generate fresh self-play WDL" "pwsh" $datagenArgs

    Invoke-Checked "extract independent phase-balanced confirmation" "python" @(
        "tools/texel/extract_parallel.py", $pgn,
        "--out-dir", $datasetDir,
        "--target-train", "127778",
        "--validation-pct", "0",
        "--test-pct", "50",
        "--jobs", [string]$Jobs
    )

    $manifestPath = Join-Path $datasetDir "manifest.json"
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    if ($manifest.schema -ne "rarog-hce-wdl-v2" -or
        $manifest.label -ne "white-perspective self-play WDL" -or
        [double]$manifest.train_blend -ne 1.0) {
        throw "confirmation dataset is not pure self-play WDL"
    }
    if ([int]$manifest.independent_starts -ne $Games -or
        [int]$manifest.recorded_games -ne $Games -or
        [int]$manifest.parse_errors -ne 0 -or
        [int]$manifest.paired_replays_discarded -ne 0) {
        throw "confirmation dataset does not contain exactly $Games clean independent starts"
    }
    if ([int]$manifest.rows.train -ne 127778 -or
        [int]$manifest.rows.validation -ne 0 -or
        [int]$manifest.rows.test -ne 127778) {
        throw "confirmation dataset row counts differ from the frozen 127778/0/127778 contract"
    }
    if ($manifest.label_contract.adjudication.Name -ne $ExpectedAdjudication -or
        [int]$manifest.inputs[0].book_start -ne $Start -or
        [int]$manifest.inputs[0].book_end -ne ($Start + $Games - 1)) {
        throw ("confirmation provenance does not identify the frozen unused-opening " +
               "segment under label contract '$ExpectedAdjudication' (manifest says " +
               "'$($manifest.label_contract.adjudication.Name)')")
    }
    $testPath = Join-Path $datasetDir "test.csv"
    $testAudit = Get-WdlCsvAudit $testPath
    if ($testAudit.Rows -ne 127778) { throw "test.csv has $($testAudit.Rows) rows, expected 127778" }
    if ((Get-HarnessSha256 $testPath) -ne [string]$manifest.output_sha256.test) {
        throw "test.csv hash differs from its publication manifest"
    }

    Invoke-Checked "build exact evaluator" "cargo" @("build", "--release", "-p", "texel-tuner")
    $tuner = Join-Path $repo "target/release/rarog-texel.exe"
    Invoke-Checked "save exact source vector" $tuner @("--write-defaults", $sourceVector)

    Write-Host "`n== one-shot exact source-to-rounded-candidate confirmation =="
    $process = Start-Process -FilePath $tuner -WindowStyle Hidden -Wait -PassThru `
        -RedirectStandardOutput $compareLog -RedirectStandardError (Join-Path $runDir "exact-confirmation.stderr.log") `
        -ArgumentList @(
            "--compare-frozen", $testPath, $sourceVector, $candidateVector, $testMarker,
            "--fix-k", $fixedKText
        )
    if ($process.ExitCode -ne 0) {
        throw "exact confirmation failed with exit code $($process.ExitCode); see $compareLog"
    }
    $compareText = Get-Content -LiteralPath $compareLog -Raw
    Write-Host $compareText
    $lossMatch = [regex]::Match(
        $compareText,
        'Frozen test loss = ([0-9.]+) \(source baseline ([0-9.]+), delta ([+-][0-9.]+)\)'
    )
    if (-not $lossMatch.Success) { throw "could not parse exact confirmation result" }

    $summary = [ordered]@{
        schema = "rarog-hce-confirmation-v1"
        status = "complete"
        run_dir = $runDir
        source_fit = $fitDir
        fixed_k = $fixedK
        openings = [ordered]@{ start = $Start; end = $Start + $Games - 1; games = $Games }
        positions = [ordered]@{ train_unused = 127778; validation = 0; test = 127778 }
        labels = "white-perspective self-play WDL"
        exact_test = [ordered]@{
            source_loss = [double]::Parse($lossMatch.Groups[2].Value, $invariant)
            candidate_loss = [double]::Parse($lossMatch.Groups[1].Value, $invariant)
            delta = [double]::Parse($lossMatch.Groups[3].Value, $invariant)
        }
        hashes = [ordered]@{
            candidate_vector = Get-HarnessSha256 $candidateVector
            source_vector = Get-HarnessSha256 $sourceVector
            pgn = Get-HarnessSha256 $pgn
            dataset_manifest = Get-HarnessSha256 $manifestPath
            test_csv = Get-HarnessSha256 $testPath
            test_marker = Get-HarnessSha256 $testMarker
        }
        wdl_counts = $testAudit.Counts
        strength_verdict = "not run; review before prospective SPRT registration"
    }
    Write-JsonAtomic -Path (Join-Path $runDir "summary.json") -Value $summary
    Write-Host "`nCOMPLETE: $runDir"
    Write-Host "Exact confirmation: candidate $($summary.exact_test.candidate_loss), source $($summary.exact_test.source_loss), delta $($summary.exact_test.delta)"
} finally {
    if ($transcribing) { Stop-Transcript | Out-Null }
}
