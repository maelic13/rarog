# Build N INDEPENDENT PGO binaries of one ISA tier, for pooled NPS comparison.
#
# Why this exists: a single PGO build carries a fixed per-binary offset of about
# 0.4% (RAR-P17, RAR-P18), so `nps_multibuild.ps1` needs several builds per arm
# to resolve a sub-1% effect. Producing them by hand invites the two failure
# modes this file exists to prevent:
#
#   1. Measuring the same bytes twice. `cargo xtask build --pgo` overwrites one
#      asset path, so copies must be taken between builds. If two copies hash
#      the same, the PGO profile did not actually vary and pooling them is
#      meaningless - this script FAILS instead of silently averaging duplicates.
#   2. Measuring the wrong binary. Every copy is launched and must reproduce the
#      expected bench fingerprint before it is accepted into the pool.
#
# Output: $OutDir\<tier>-<n>.exe plus a manifest.txt of hashes and fingerprints.

param(
    [Parameter(Mandatory = $true)][ValidateSet("base", "x86-64", "avx2", "pext")][string]$Arch,
    [int]$Builds = 4,
    [Parameter(Mandatory = $true)][string]$OutDir,
    [int]$ExpectFingerprint = 7601220,
    [int]$Depth = 13,
    # Local experimentation only. A pool built from an unidentifiable tree must
    # never back a recorded result; `sprt.ps1` carries the same escape hatch.
    [switch]$AllowDirty
)

$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$sha = (& git rev-parse HEAD).Trim()
$dirty = (& git status --porcelain)
if ($dirty -and -not $AllowDirty) {
    throw "working tree is dirty - refusing to build a pool whose source cannot be identified (-AllowDirty to override locally)"
}
if ($dirty) { Write-Warning "DIRTY TREE: this pool is not reproducible and must not back a recorded result" }

$benchIn = Join-Path $env:TEMP "nps_pool_bench_in.txt"
Set-Content -LiteralPath $benchIn -Value "bench $Depth" -Encoding ascii

$rows = @()
for ($i = 1; $i -le $Builds; $i++) {
    Write-Host "== $Arch build $i/$Builds =="
    $log = Join-Path $OutDir "build-$Arch-$i.log"
    & cargo xtask build --arch $Arch --pgo *> $log
    if ($LASTEXITCODE -ne 0) { throw "build $i failed (exit $LASTEXITCODE); see $log" }

    # Take the artifact path from xtask itself rather than reconstructing the
    # asset name here - the naming rules live in one place and must stay there.
    $built = (Select-String -Path $log -Pattern '^Built (.+)$' | Select-Object -Last 1)
    if (-not $built) { throw "no 'Built <path>' line in $log - cannot identify the artifact" }
    $src = $built.Matches[0].Groups[1].Value.Trim()
    if (-not (Test-Path $src)) { throw "xtask reported '$src' but it does not exist" }

    $dst = Join-Path $OutDir "$Arch-$i.exe"
    Copy-Item $src $dst -Force

    $hash = (Get-FileHash -Algorithm SHA256 $dst).Hash
    $outFile = Join-Path $env:TEMP "nps_pool_bench_out.txt"
    $p = Start-Process -FilePath $dst -RedirectStandardInput $benchIn `
        -RedirectStandardOutput $outFile -NoNewWindow -PassThru -Wait
    if ($p.ExitCode -ne 0) { throw "$dst exited $($p.ExitCode) running bench $Depth" }
    $fpLine = Select-String -Path $outFile -Pattern '^Nodes searched\s*:\s*(\d+)' | Select-Object -Last 1
    if (-not $fpLine) { throw "no fingerprint from $dst - it benched nothing" }
    $nodes = [int]$fpLine.Matches[0].Groups[1].Value
    if ($nodes -ne $ExpectFingerprint) {
        throw "$dst benched $nodes, expected $ExpectFingerprint - this is not the binary you think it is"
    }

    "  {0}  {1}  nodes {2}" -f (Split-Path $dst -Leaf), $hash.Substring(0, 16), $nodes | Write-Host
    $rows += [pscustomobject]@{ File = $dst; Hash = $hash; Nodes = $nodes }
}

# The check that makes pooling meaningful: independent profiles => distinct bytes.
$dupes = $rows | Group-Object Hash | Where-Object { $_.Count -gt 1 }
if ($dupes) {
    $names = ($dupes.Group.File | Split-Path -Leaf) -join ", "
    throw "IDENTICAL binaries in the pool ($names) - the PGO profile did not vary, so pooling them measures nothing"
}

$manifest = Join-Path $OutDir "manifest-$Arch.txt"
@(
    "arch          : $Arch"
    "source        : $sha (clean)"
    "builds        : $Builds"
    "fingerprint   : $ExpectFingerprint (verified on every copy)"
    "generated     : $(Get-Date -Format o)"
    ""
) + ($rows | ForEach-Object { "{0}  {1}" -f $_.Hash, (Split-Path $_.File -Leaf) }) |
Set-Content -LiteralPath $manifest -Encoding ascii

Write-Host ""
Write-Host "pool OK: $Builds distinct $Arch builds, all at fingerprint $ExpectFingerprint"
Write-Host "manifest: $manifest"
Write-Host ("set for nps_multibuild: " + (($rows.File | ForEach-Object { "`"$_`"" }) -join ", "))
