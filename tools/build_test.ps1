<#
.SYNOPSIS
    Build a Rarog test binary and copy it to the test-engines folder.

.DESCRIPTION
    Three modes:

    Normal (default): runs `cargo xtask build --arch pext --pgo` — produces a
    PGO-optimised pext binary.  Use for SPRT and gauntlet testing.  PGO changes
    hot-path timing enough to affect measured Elo comparisons; always use this
    for match testing.

    Native (-Native switch): runs `cargo xtask build --arch pext --native --pgo` —
    produces a PGO-optimised binary built with `-C target-cpu=native` for the
    exact host CPU (e.g. znver3 on a 5950X), instead of the portable
    x86-64-v3 baseline.  Use for local/own-match testing and deployment on the
    machine that built it; do NOT distribute this binary, since it is not
    guaranteed to run on other CPUs.

    Tune (-Tune switch): runs `cargo build --release --features tune` — produces
    a non-PGO pext binary with search-parameter UCI options exposed.  Use ONLY
    for weather-factory SPSA runs.  PGO is skipped because (a) xtask does not
    support --features, and (b) SPSA accuracy does not depend on absolute NPS —
    both sides of each mini-match use the same binary.

    Output always goes to tools\test_engines\ (repo-local and separate from
    released engines).

    A schema-versioned sidecar binds the copied binary to its exact SHA-256,
    source tree, compiler, build command and bench verification. Use
    -SourceRoot to build a frozen worktree with this same checked wrapper.

.PARAMETER Suffix
    Short label for the output file.
    Normal:  rarog-<Suffix>-pext-pgo.exe
    Native:  rarog-<Suffix>-pext-native-pgo.exe
    Tune:    rarog-<Suffix>-tune.exe

.PARAMETER Native
    Build with `--arch pext --native --pgo` instead of `--arch pext --pgo`.
    Same PEXT code path; only the codegen baseline changes.  Local-only.

.PARAMETER Tune
    Build with --features tune instead of PGO.  Use for SPSA binaries only.

.PARAMETER TestEnginesDir
    Destination directory.  Default: tools\test_engines

.PARAMETER SourceRoot
    Rarog worktree to build. Default: the repository containing this script.

.PARAMETER BuildOnly
    Write a hash-bound manifest without launching bench. Match and datagen
    launchers reject this verification state; it is only for staged builds.

.EXAMPLE
    # Normal SPRT binary
    ./tools/build_test.ps1 -Suffix phase1-lmr

.EXAMPLE
    # Native (-march=native-equivalent) binary for local-only testing
    ./tools/build_test.ps1 -Suffix phase292-native -Native

.EXAMPLE
    # SPSA tuning binary (exposes UCI options)
    ./tools/build_test.ps1 -Suffix phase1-lmr -Tune
#>
param(
    [Parameter(Mandatory)][string]$Suffix,
    [switch]$Tune,
    [switch]$Native,
    [switch]$BuildOnly,
    [int]$BenchDepth = 13,
    [string]$TestEnginesDir = "$PSScriptRoot\test_engines",
    [string]$SourceRoot = ""
)

if ($Tune -and $Native) {
    throw "-Tune and -Native are mutually exclusive."
}
if ($BenchDepth -lt 1) { throw "-BenchDepth must be positive." }

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
. "$PSScriptRoot\harness_common.ps1"

# --- 9.7 provenance manifest -------------------------------------------------
# Every test binary gets a sidecar JSON next to it: git SHA + dirty flag,
# branch, rustc, and a bench fingerprint VERIFIED by running the binary just
# built (which doubles as a smoke test — a broken build fails here, not in an
# SPRT). sprt.ps1 copies both engines' manifests into the result dir, so every
# result is permanently self-describing.
#
# LOCAL-ONLY BY DESIGN (user decision 2026-07-20): manifests exist for
# development provenance. tools/test_engines/ and tools/results/ are
# gitignored, and the release workflow (build.yml) has NO manifest step —
# nothing here can ever appear on the GitHub release page.
function Write-EngineManifest {
    param(
        [Parameter(Mandatory)][string]$BinaryPath,
        [Parameter(Mandatory)][string]$Suffix,
        [Parameter(Mandatory)][string]$Flavor,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$BuildCommand,
        [Parameter(Mandatory)][int]$Depth,
        [switch]$SkipBench
    )

    $sha    = (& git -C $RepositoryRoot rev-parse HEAD).Trim()
    $tree   = (& git -C $RepositoryRoot rev-parse 'HEAD^{tree}').Trim()
    $branch = (& git -C $RepositoryRoot rev-parse --abbrev-ref HEAD).Trim()
    $dirty  = [bool](& git -C $RepositoryRoot status --porcelain)
    $rustc  = (rustc -V).Trim()
    $binary = Get-Item -LiteralPath $BinaryPath

    $nodes = $null
    $benchLine = $null
    if (-not $SkipBench) {
        Write-Host "Verifying bench fingerprint of $($binary.Name) (depth $Depth) ..."
        $inputPath = [IO.Path]::GetTempFileName()
        $stdoutPath = [IO.Path]::GetTempFileName()
        $stderrPath = [IO.Path]::GetTempFileName()
        try {
            Set-Content -LiteralPath $inputPath -Value "bench $Depth" -Encoding ascii
            $process = Start-Process -FilePath $BinaryPath -WindowStyle Hidden -Wait -PassThru `
                -RedirectStandardInput $inputPath -RedirectStandardOutput $stdoutPath `
                -RedirectStandardError $stderrPath
            if ($process.ExitCode -ne 0) {
                throw "Bench exited with code $($process.ExitCode); refusing an unverified manifest."
            }
            $benchOut = (Get-Content -LiteralPath $stdoutPath -Raw) +
                (Get-Content -LiteralPath $stderrPath -Raw)
        } finally {
            Remove-Item -LiteralPath $inputPath, $stdoutPath, $stderrPath -Force -ErrorAction SilentlyContinue
        }
        $benchLine = (($benchOut -split "`n" | Where-Object { $_ -match "Nodes searched" }) | Select-Object -Last 1).Trim()
        if ($benchLine -notmatch "([0-9][0-9,]*)\s*$") {
            throw "Could not parse a bench node count from the built binary - refusing an unverified manifest."
        }
        $nodes = [int64]($Matches[1] -replace ",", "")
        if ($nodes -le 0) { throw "Bench reported $nodes nodes - broken binary." }
    }

    $manifest = [ordered]@{
        schema_version    = 2
        engine            = $binary.Name
        binary_sha256     = Get-HarnessSha256 $BinaryPath
        binary_size_bytes = $binary.Length
        suffix            = $Suffix
        flavor            = $Flavor
        build_command      = $BuildCommand
        git_sha           = $sha
        git_tree          = $tree
        git_branch        = $branch
        git_dirty         = $dirty
        rustc             = $rustc
        verification      = if ($SkipBench) { "build-only" } else { "bench" }
        bench_depth       = if ($SkipBench) { $null } else { $Depth }
        bench_nodes       = $nodes
        bench_line        = $benchLine
        pgo_workload      = if ($Flavor -like "*-pgo") { "bench 13 (xtask default)" } else { $null }
        built_utc         = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    }

    $manifestPath = [IO.Path]::ChangeExtension($BinaryPath, ".json")
    Write-JsonAtomic -Path $manifestPath -Value $manifest
    $verified = if ($SkipBench) { "build-only; engine not launched" } else { "bench $nodes" }
    Write-Host "Manifest: $manifestPath  ($verified$(if ($dirty) { ', DIRTY WORKING TREE' }))"
    if ($dirty) {
        Write-Host "WARNING: built from a DIRTY working tree — this binary is not reproducible from git_sha alone." -ForegroundColor Yellow
    }
}


$repoRoot = if ($SourceRoot) {
    (Resolve-Path -LiteralPath $SourceRoot).Path
} else {
    Split-Path -Parent $PSScriptRoot
}
if (-not (Test-Path -LiteralPath (Join-Path $repoRoot "Cargo.toml") -PathType Leaf)) {
    throw "-SourceRoot is not a Rarog worktree: $repoRoot"
}
$TestEnginesDir = [IO.Path]::GetFullPath($TestEnginesDir)
Push-Location $repoRoot
try {
    if ($Tune) {
        Write-Host ""
        Write-Host "Building pext tune binary (--features tune, no PGO) — suffix: $Suffix"
        Write-Host "NOTE: Use this binary only for SPSA, never for SPRT."
        Write-Host ""

        # pext RUSTFLAGS matching xtask's pext arch (rarog_pext cfg + BMI2 target features).
        $savedRustFlags = $env:RUSTFLAGS
        try {
            $env:RUSTFLAGS = "--cfg rarog_pext -C target-cpu=x86-64-v3 -C target-feature=+bmi2"
            cargo build --release --features tune
            if ($LASTEXITCODE -ne 0) { throw "cargo build --features tune failed (exit $LASTEXITCODE)" }
        } finally {
            $env:RUSTFLAGS = $savedRustFlags
        }

        $src = Join-Path $repoRoot "target\release\rarog.exe"
        if (-not (Test-Path $src)) { throw "Binary not found at: $src" }

        if (-not (Test-Path $TestEnginesDir)) {
            New-Item -ItemType Directory -Path $TestEnginesDir | Out-Null
        }

        $dest = Join-Path $TestEnginesDir "rarog-$Suffix-tune.exe"
        Copy-Item $src $dest -Force
        Write-EngineManifest -BinaryPath $dest -Suffix $Suffix -Flavor "pext-tune" `
            -RepositoryRoot $repoRoot -BuildCommand "cargo build --release --features tune" `
            -Depth $BenchDepth -SkipBench:$BuildOnly
        Write-Host ""
        Write-Host "Done: $dest"
        Write-Host ""
    } else {
        # 2.3.0: `--native` is now ORTHOGONAL to `--arch`. Both flavours build
        # the PEXT code path; -Native only swaps the portable x86-64-v3 baseline
        # for `target-cpu=native`. Gate binaries deliberately stay portable, so
        # what we SPRT matches the shipped pext asset (PLAN S3).
        $arch = "pext"
        $label = if ($Native) { "pext+native" } else { "pext" }
        Write-Host ""
        Write-Host "Building $label+PGO binary (suffix: $Suffix) ..."
        Write-Host ""

        if ($Native) {
            cargo xtask build --arch $arch --native --pgo
        } else {
            cargo xtask build --arch $arch --pgo
        }
        if ($LASTEXITCODE -ne 0) { throw "xtask build failed (exit $LASTEXITCODE)" }

        $dist = Get-ChildItem "target/dist/rarog-*-$arch-pgo.exe" |
            Sort-Object LastWriteTime -Descending |
            Select-Object -First 1

        if (-not $dist) {
            throw "No $arch-pgo binary found in target/dist/ — check xtask output above."
        }

        if (-not (Test-Path $TestEnginesDir)) {
            New-Item -ItemType Directory -Path $TestEnginesDir | Out-Null
        }

        $fileFlavor = if ($Native) { "$arch-native-pgo" } else { "$arch-pgo" }
        $dest = Join-Path $TestEnginesDir "rarog-$Suffix-$fileFlavor.exe"
        Copy-Item $dist.FullName $dest -Force
        $buildCommand = "cargo xtask build --arch $arch$(if ($Native) { ' --native' }) --pgo"
        Write-EngineManifest -BinaryPath $dest -Suffix $Suffix -Flavor $fileFlavor `
            -RepositoryRoot $repoRoot -BuildCommand $buildCommand -Depth $BenchDepth `
            -SkipBench:$BuildOnly
        Write-Host ""
        Write-Host "Done: $dest"
        Write-Host ""
    }
} finally {
    Pop-Location
}
