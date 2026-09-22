# Audit declared search tunables against the reusable SPSA configurations.
#
# The configs are historical/reusable surfaces, not an instruction to launch a
# tune. This audit catches the expensive mechanical failures before any future
# experiment is registered: stale names, stale seeds, invalid ranges, binary
# coordinates and integer perturbations that round to zero before the horizon.

$ErrorActionPreference = "Stop"
$repo = Split-Path $PSScriptRoot -Parent
$horizon = 5000
$gamma = 0.102

$defaults = @{}
$declared = @{}
foreach ($m in (Select-String -Path (Join-Path $repo "src\search\params.rs") `
            -Pattern '^\s+\w+ = (-?[\d_]+), "(\w+)", (-?[\d_]+)\.\.=(-?[\d_]+)').Matches) {
    $name = $m.Groups[2].Value
    $defaults[$name] = [int]($m.Groups[1].Value -replace '_', '')
    $declared[$name] = [pscustomobject]@{
        Min = [int]($m.Groups[3].Value -replace '_', '')
        Max = [int]($m.Groups[4].Value -replace '_', '')
    }
}

# A historical group is a surface whose tune has been baked: its seeds are the
# registration's, frozen as evidence, and no longer the engine's defaults, so
# classes 2 and 3 do not apply to it. Groups are listed one per line in
# tools/spsa_configs/historical.txt; everything else is live.
$historicalPath = Join-Path $repo "tools\spsa_configs\historical.txt"
$historical = @()
if (Test-Path -LiteralPath $historicalPath) {
    $historical = @(Get-Content -LiteralPath $historicalPath | ForEach-Object { ($_ -split '#')[0].Trim() } | Where-Object { $_ })
}

$groups = @{}
foreach ($file in Get-ChildItem (Join-Path $repo "tools\spsa_configs\config_*.json")) {
    $group = $file.BaseName -replace '^config_', ''
    $json = Get-Content $file.FullName -Raw | ConvertFrom-Json
    foreach ($parameter in $json.PSObject.Properties) {
        if (-not $groups.ContainsKey($parameter.Name)) { $groups[$parameter.Name] = @() }
        $groups[$parameter.Name] += [pscustomobject]@{
            Group = $group
            Value = [int]$parameter.Value.value
            Min = [int]$parameter.Value.min_value
            Max = [int]$parameter.Value.max_value
            Step = [double]$parameter.Value.step
        }
    }
}

"declared tunables: $($defaults.Count)    names across reusable SPSA groups: $($groups.Count)"
"historical groups (classes 2 and 3 not applied): $(if ($historical) { $historical -join ', ' } else { 'none' })"
$problems = 0

""
"== 1. declared but in NO reusable SPSA group =="
$orphans = $defaults.Keys | Where-Object { -not $groups.ContainsKey($_) } | Sort-Object
if ($orphans) { $orphans | ForEach-Object { "   $_" } } else { "   none" }
"   (expected for accepted constants, categorical gates and future-owned mechanisms)"

""
"== 2. in a live group but NOT declared (ERROR) =="
$stale = $groups.Keys | Where-Object {
    -not $defaults.ContainsKey($_) -and @($groups[$_] | Where-Object { $historical -notcontains $_.Group }).Count -gt 0
} | Sort-Object
if ($stale) {
    $stale | ForEach-Object { "   $_"; $problems++ }
} else { "   none" }

""
"== 3. live seed disagrees with baked default (ERROR) =="
$drift = 0
foreach ($name in ($groups.Keys | Sort-Object)) {
    if (-not $defaults.ContainsKey($name)) { continue }
    foreach ($entry in $groups[$name]) {
        if ($historical -contains $entry.Group) { continue }
        if ($entry.Value -ne $defaults[$name]) {
            "   {0,-22} {1,-12} seed {2,8} vs default {3,8}" -f `
                $name, $entry.Group, $entry.Value, $defaults[$name]
            $drift++
        }
    }
}
if ($drift -eq 0) { "   none" } else { $problems += $drift }

""
"== 4. present in more than one group (information) =="
$multi = $groups.Keys | Where-Object {
    @($groups[$_].Group | Select-Object -Unique).Count -gt 1
} | Sort-Object
if ($multi) {
    $multi | ForEach-Object {
        "   {0,-22} -> {1}" -f $_, (($groups[$_].Group | Select-Object -Unique) -join ', ')
    }
} else { "   none" }

""
"== 5. invalid range, pin or categorical coordinate (ERROR) =="
$invalid = 0
foreach ($name in ($groups.Keys | Sort-Object)) {
    foreach ($entry in $groups[$name]) {
        $span = $entry.Max - $entry.Min
        $bad = $entry.Min -gt $entry.Max -or $entry.Value -lt $entry.Min -or `
            $entry.Value -gt $entry.Max -or $entry.Step -le 0
        if (-not $bad -and $declared.ContainsKey($name)) {
            $engine = $declared[$name]
            $bad = $entry.Min -lt $engine.Min -or $entry.Max -gt $engine.Max
        }
        if ($bad -or $span -le 1) {
            "   {0,-22} {1,-12} value={2} [{3}..{4}] step={5}" -f `
                $name, $entry.Group, $entry.Value, $entry.Min, $entry.Max, $entry.Step
            $invalid++
        }
    }
}
if ($invalid -eq 0) { "   none" } else { $problems += $invalid }

""
"== 6. perturbation rounds to zero before N=$horizon (ERROR) =="
$dead = 0
$cEnd = 1.0 / [Math]::Pow($horizon, $gamma)
foreach ($name in ($groups.Keys | Sort-Object)) {
    foreach ($entry in $groups[$name]) {
        $perturbation = $entry.Step * $cEnd
        if ($perturbation -lt 0.5) {
            $deadAt = [Math]::Pow(2.0 * $entry.Step, 1.0 / $gamma)
            "   {0,-22} {1,-12} step={2} end={3:N2}; dead from {4:N0}" -f `
                $name, $entry.Group, $entry.Step, $perturbation, $deadAt
            $dead++
        }
    }
}
if ($dead -eq 0) { "   none" } else { $problems += $dead }

""
"== 7. seed on a rail (WARNING) =="
$rails = 0
foreach ($name in ($groups.Keys | Sort-Object)) {
    foreach ($entry in $groups[$name]) {
        if ($entry.Value -le $entry.Min -or $entry.Value -ge $entry.Max) {
            "   {0,-22} {1,-12} value={2} [{3}..{4}]" -f `
                $name, $entry.Group, $entry.Value, $entry.Min, $entry.Max
            $rails++
        }
    }
}
if ($rails -eq 0) { "   none" } else { "   $rails one-sided seed(s)" }

""
if ($problems -eq 0) {
    "RESULT: clean"
} else {
    throw "RESULT: $problems SPSA coverage issue(s) needing attention"
}
