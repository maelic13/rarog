# Shared preflight for clock-based fastchess harnesses.

$script:MinimumAffinityFastchessVersion = [version]"1.7.0"
$script:HarnessIsWindows = [Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT

# One named source of truth for result adjudication in strength measurements.
#
# Both profiles are 600/3 TWO-SIDED as of 2026-08-18, by maintainer decision:
# one rule everywhere is worth more than the small saving one-sided bought.
# Two-sided requires both engines to agree before a game is called, so this is
# the CONSERVATIVE direction -- fewer adjudications, more games played out,
# marginally more wall time.
#
# The 2026-08-02 calibration that had split them is retained here as evidence,
# not as policy: over 69,350 Rarog games completed under two-sided 600/3,
# one-sided 600/3 produced no chess-result reversals (three apparent reversals
# were later time forfeits) and changed 71 results to wins that later drew,
# 0.20% of its 35,486 triggers. That is what makes unifying cheap -- the two
# rules were measured to differ on 0.20% of triggers and never on a final
# chess result -- and it is also why unifying is safe rather than merely tidy.
#
# Historical note for anyone reading old ledger rows: strength results before
# 2026-08-18 were adjudicated one-sided. The 0.20% figure above is the measured
# size of that discontinuity.
function Get-StrengthTestProfile {
    [pscustomobject]@{
        Name               = "strength-v2"
        ResignMoveCount    = 3
        ResignScore        = 600
        ResignTwoSided     = $true
        DrawMoveNumber     = 40
        DrawMoveCount      = 8
        DrawScore          = 10
    }
}

function Get-StrengthTestResignArgs {
    $profile = Get-StrengthTestProfile
    $args = @(
        '-resign'
        "movecount=$($profile.ResignMoveCount)"
        "score=$($profile.ResignScore)"
    )
    if ($profile.ResignTwoSided) { $args += 'twosided=true' }
    $args
}

# Datagen keeps its own named profile even though the values now match
# strength-v2 exactly. The reason is ownership, not arithmetic: a false
# resignation assigns the wrong target to every position sampled from that
# game, so if strength adjudication is ever loosened again, labels must not
# follow it silently. Same numbers today, different owner and different
# justification.
function Get-DatagenProfile {
    [pscustomobject]@{
        Name               = "datagen-v1"
        ResignMoveCount    = 3
        ResignScore        = 600
        ResignTwoSided     = $true
        DrawMoveNumber     = 40
        DrawMoveCount      = 8
        DrawScore          = 10
    }
}

# datagen-v2: no adjudication (PROCESS.md "Adjudication", RAR-M17); datagen-v1 stays by name for its manifests.
function Get-DatagenProfileV2 {
    [pscustomobject]@{
        Name               = "datagen-v2"
        Adjudication       = $false
        ResignMoveCount    = $null
        ResignScore        = $null
        ResignTwoSided     = $false
        DrawMoveNumber     = $null
        DrawMoveCount      = $null
        DrawScore          = $null
    }
}

# datagen-v3: no eval adjudication, Syzygy truth at 6 men with the fifty-move rule kept (RAR-M18).
# DATAGEN ONLY: a strength gate measures realized conversion, which tablebase adjudication would erase.
function Get-DatagenProfileV3 {
    param([Parameter(Mandatory)][string]$SyzygyPath, [int]$Pieces = 6)
    [pscustomobject]@{
        Name               = "datagen-v3"
        Adjudication       = $false
        TablebaseAdjudication = $true
        TablebasePath      = $SyzygyPath
        TablebasePieces    = $Pieces
        TablebaseIgnore50  = $false
        ResignMoveCount    = $null
        ResignScore        = $null
        ResignTwoSided     = $false
        DrawMoveNumber     = $null
        DrawMoveCount      = $null
        DrawScore          = $null
    }
}

function Get-DatagenResignArgs {
    $profile = Get-DatagenProfile
    $args = @(
        '-resign'
        "movecount=$($profile.ResignMoveCount)"
        "score=$($profile.ResignScore)"
    )
    if ($profile.ResignTwoSided) { $args += 'twosided=true' }
    $args
}

function Get-HarnessPhysicalCpus {
    if ($script:HarnessIsWindows) {
        if (-not ('RarogHarness.CpuTopology' -as [type])) {
            Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Runtime.InteropServices;

namespace RarogHarness {
    public sealed class CpuCore {
        public int Cpu { get; set; }
        public int EfficiencyClass { get; set; }
    }

    public static class CpuTopology {
        private const int RelationProcessorCore = 0;

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool GetLogicalProcessorInformationEx(
            int relationship, IntPtr buffer, ref uint returnedLength);

        public static CpuCore[] PhysicalCpus() {
            uint length = 0;
            GetLogicalProcessorInformationEx(RelationProcessorCore, IntPtr.Zero, ref length);
            if (length == 0) throw new Win32Exception(Marshal.GetLastWin32Error());

            IntPtr buffer = Marshal.AllocHGlobal((int)length);
            try {
                if (!GetLogicalProcessorInformationEx(RelationProcessorCore, buffer, ref length))
                    throw new Win32Exception(Marshal.GetLastWin32Error());

                var result = new List<CpuCore>();
                int offset = 0;
                int groupAffinitySize = IntPtr.Size + 8;
                while (offset < length) {
                    IntPtr entry = IntPtr.Add(buffer, offset);
                    int relationship = Marshal.ReadInt32(entry, 0);
                    int size = Marshal.ReadInt32(entry, 4);
                    if (size <= 0 || offset + size > length)
                        throw new InvalidOperationException("Invalid Windows CPU-topology record.");

                    if (relationship == RelationProcessorCore) {
                        int efficiencyClass = Marshal.ReadByte(entry, 9);
                        int groupCount = (ushort)Marshal.ReadInt16(entry, 30);
                        var logical = new List<int>();
                        for (int groupIndex = 0; groupIndex < groupCount; ++groupIndex) {
                            int gaOffset = 32 + groupIndex * groupAffinitySize;
                            ulong mask = IntPtr.Size == 8
                                ? unchecked((ulong)Marshal.ReadInt64(entry, gaOffset))
                                : unchecked((uint)Marshal.ReadInt32(entry, gaOffset));
                            int group = (ushort)Marshal.ReadInt16(entry, gaOffset + IntPtr.Size);
                            for (int bit = 0; bit < IntPtr.Size * 8; ++bit)
                                if ((mask & (1UL << bit)) != 0) logical.Add(group * 64 + bit);
                        }
                        if (logical.Count == 0)
                            throw new InvalidOperationException("A physical core has no logical processors.");
                        result.Add(new CpuCore {
                            Cpu = logical.Min(),
                            EfficiencyClass = efficiencyClass
                        });
                    }
                    offset += size;
                }

                return result
                    .OrderByDescending(c => c.EfficiencyClass)
                    .ThenBy(c => c.Cpu)
                    .ToArray();
            } finally {
                Marshal.FreeHGlobal(buffer);
            }
        }
    }
}
'@
        }
        return [RarogHarness.CpuTopology]::PhysicalCpus()
    }

    if (Get-Command lscpu -ErrorAction SilentlyContinue) {
        $seen = @{}
        $cores = foreach ($line in (& lscpu '-p=CPU,CORE,SOCKET' 2>$null)) {
            if (-not $line -or $line.StartsWith('#')) { continue }
            $cpu, $core, $socket = $line.Split(',')
            $key = "$socket,$core"
            if (-not $seen.ContainsKey($key)) {
                $seen[$key] = $true
                [pscustomobject]@{ Cpu = [int]$cpu; EfficiencyClass = 0 }
            }
        }
        return @($cores | Sort-Object Cpu)
    }

    return @(0..([Environment]::ProcessorCount - 1) |
        ForEach-Object { [pscustomobject]@{ Cpu = $_; EfficiencyClass = 0 } })
}

function Get-FastchessVersion {
    param([Parameter(Mandatory)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) { throw "fastchess not found: $Path" }

    $line = (& $Path --version 2>&1 | Select-Object -First 1)
    if (-not $line) { throw "Could not query fastchess version at '$Path'." }

    $match = [regex]::Match("$line", '(?<major>\d+)\.(?<minor>\d+)\.(?<patch>\d+)')
    if (-not $match.Success) { throw "Unrecognized fastchess version string: '$line'." }

    [pscustomobject]@{
        Text    = "$line".Trim()
        Version = [version]::new(
            [int]$match.Groups['major'].Value,
            [int]$match.Groups['minor'].Value,
            [int]$match.Groups['patch'].Value)
    }
}

function Assert-AffinityFastchess {
    param([Parameter(Mandatory)][string]$Path)

    $info = Get-FastchessVersion -Path $Path
    if ($script:HarnessIsWindows -and $info.Version -lt $script:MinimumAffinityFastchessVersion) {
        throw "fastchess $($info.Version) is too old for reliable Windows affinity. " +
              "Version 1.7.0 contains the process-affinity fix; run tools/setup_tools.ps1 " +
              "to install the pinned runner. Found: $($info.Text)"
    }
    $info
}

function Get-HarnessGameCpus {
    # The cores a TIMED game may be pinned to: every physical core except the
    # lowest-numbered one. Windows services most device interrupts and their
    # deferred procedure calls on CPU 0, so an engine pinned there inherits
    # stalls no other core sees. The reserve in Resolve-HarnessConcurrency
    # already leaves cores free; this makes one of them CPU 0 by construction
    # instead of whichever cores happen to sort last.
    $cores = @(Get-HarnessPhysicalCpus)
    if ($cores.Count -le 1) { return $cores }
    @($cores | Sort-Object Cpu | Select-Object -Skip 1)
}

function Get-PhysicalCoreCount {
    $count = @(Get-HarnessPhysicalCpus).Count
    if (-not $count -or $count -lt 1) { $count = 1 }
    [int]$count
}

function Resolve-HarnessConcurrency {
    # `ThreadsPerGame` generalises this past the 1-thread assumption.
    # Each concurrent game needs `ThreadsPerGame` physical cores, so the core
    # budget is divided, not handed out one game per core. At Threads=1 the
    # arithmetic is identical to before, so 1-thread runs are unaffected.
    # `AllowOversubscribe` is for NODE-LIMITED work only, and datagen is the
    # only caller that qualifies. The physical-core ceiling exists because a
    # TIMED game mismeasures under contention -- that is the whole affinity and
    # forfeit story, RAR-M14 included. A `go nodes` search has no such exposure:
    # it plays identical moves however slowly it runs, so contention costs wall
    # time and changes nothing else. Holding datagen to 14 of 32 logical
    # processors therefore threw away throughput for a hazard it does not have.
    param([int]$Requested, [int]$ReservePhysicalCores = 2, [int]$ThreadsPerGame = 1,
          [switch]$AllowOversubscribe)

    if ($ThreadsPerGame -lt 1) { throw "ThreadsPerGame must be >= 1 (got $ThreadsPerGame)." }
    $physical = Get-PhysicalCoreCount
    # A timed game is never pinned to CPU 0 (Get-HarnessGameCpus), so the timed
    # ceiling is the game-core count, one less than the physical count.
    $ceiling = if ($AllowOversubscribe) { [Environment]::ProcessorCount } else { @(Get-HarnessGameCpus).Count }
    $budgetBase = if ($AllowOversubscribe) { $ceiling } else { $physical }
    $budget = [Math]::Max(1, $budgetBase - $ReservePhysicalCores)
    $recommended = [Math]::Max(1, [Math]::Floor($budget / $ThreadsPerGame))
    $resolved = if ($Requested -gt 0) { $Requested } else { $recommended }
    $needed = $resolved * $ThreadsPerGame
    if ($needed -gt $ceiling) {
        $kind = if ($AllowOversubscribe) { "logical processors" } else { "game cores (physical cores except CPU 0)" }
        throw ("Concurrency $resolved x Threads $ThreadsPerGame = $needed, " +
               "which exceeds the detected $ceiling $kind.")
    }
    [pscustomobject]@{
        Concurrency   = [int]$resolved
        PhysicalCores = [int]$physical
        CoresUsed     = [int]$needed
        ThreadsPerGame = [int]$ThreadsPerGame
        AutoSelected  = ($Requested -le 0)
    }
}

function Get-HarnessAffinityCpuList {
    # The pinned set must cover EVERY core the games will use, i.e.
    # Concurrency x ThreadsPerGame — not one core per game. Under-sizing this
    # list silently oversubscribes cores and reintroduces exactly the hidden
    # per-run offset the affinity pinning exists to remove.
    param([Parameter(Mandatory)][int]$Concurrency, [int]$ThreadsPerGame = 1)

    $cores = @(Get-HarnessGameCpus)
    $needed = $Concurrency * $ThreadsPerGame
    if ($needed -gt $cores.Count) {
        throw "Concurrency $Concurrency x Threads $ThreadsPerGame = $needed exceeds $($cores.Count) game cores (physical cores except CPU 0)."
    }
    (($cores | Select-Object -First $needed).Cpu -join ',')
}

function New-HarnessSeed {
    param([int]$Requested)
    if ($Requested -ne 0) { return $Requested }
    Get-Random -Minimum 1 -Maximum ([int]::MaxValue)
}

function Get-HarnessSha256 {
    param([Parameter(Mandatory)][string]$Path)
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
}

function Get-EngineUciOptions {
    param(
        [Parameter(Mandatory)][string]$Path,
        [int]$TimeoutMs = 15000,
        [switch]$Detailed
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Engine not found: $Path"
    }

    $full = (Resolve-Path -LiteralPath $Path).Path
    $psi = [System.Diagnostics.ProcessStartInfo]::new()
    $psi.FileName               = $full
    $psi.WorkingDirectory       = Split-Path -Parent $full
    $psi.RedirectStandardInput  = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError  = $true
    $psi.UseShellExecute        = $false

    $proc = [System.Diagnostics.Process]::Start($psi)
    $text = ""
    try {
        # Read asynchronously before writing. Otherwise a verbose engine can
        # fill the stdout pipe while WaitForExit waits for a process that can
        # no longer make progress.
        $stdout = $proc.StandardOutput.ReadToEndAsync()
        $stderr = $proc.StandardError.ReadToEndAsync()
        $proc.StandardInput.WriteLine("uci")
        $proc.StandardInput.WriteLine("quit")
        $proc.StandardInput.Close()
        if (-not $proc.WaitForExit($TimeoutMs)) {
            throw "Engine '$Path' did not answer 'uci' within ${TimeoutMs} ms."
        }
        $text = $stdout.Result
        $errorText = $stderr.Result
        if ($proc.ExitCode -ne 0) {
            throw "Engine '$Path' exited $($proc.ExitCode) during UCI discovery: $errorText"
        }
    } finally {
        if (-not $proc.HasExited) { $proc.Kill($true) }
        $proc.Dispose()
    }

    if ($text -notmatch '(?m)^\s*uciok\s*$') {
        throw "Engine '$Path' did not emit 'uciok'; it is not a working UCI engine."
    }

    $options = [System.Collections.Generic.List[object]]::new()
    foreach ($line in ($text -split "`r?`n")) {
        $match = [regex]::Match($line, '^\s*option\s+name\s+(?<name>.+?)\s+type\s+(?<type>\S+)(?<tail>.*)$')
        if (-not $match.Success) { continue }
        $tail = $match.Groups['tail'].Value
        $defaultMatch = [regex]::Match($tail, '(?:^|\s)default\s+(?<value>\S+)')
        $minMatch = [regex]::Match($tail, '(?:^|\s)min\s+(?<value>-?\d+)')
        $maxMatch = [regex]::Match($tail, '(?:^|\s)max\s+(?<value>-?\d+)')
        $options.Add([pscustomobject]@{
            Name    = $match.Groups['name'].Value.Trim()
            Type    = $match.Groups['type'].Value
            Default = if ($defaultMatch.Success) { $defaultMatch.Groups['value'].Value } else { $null }
            Min     = if ($minMatch.Success) { [int64]$minMatch.Groups['value'].Value } else { $null }
            Max     = if ($maxMatch.Success) { [int64]$maxMatch.Groups['value'].Value } else { $null }
            Raw     = $line.Trim()
        })
    }
    if ($Detailed) { $options.ToArray(); return }
    $options.Name
}

function Test-EngineSupportsOption {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Name
    )

    $normalize = { param($value) ($value -replace '\s+', ' ').Trim().ToLowerInvariant() }
    $target = & $normalize $Name
    foreach ($advertised in (Get-EngineUciOptions -Path $Path)) {
        if ((& $normalize $advertised) -eq $target) { return $true }
    }
    $false
}

function Write-JsonAtomic {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][object]$Value,
        [int]$Depth = 8
    )

    $temporary = "$Path.tmp"
    try {
        $Value | ConvertTo-Json -Depth $Depth |
            Set-Content -LiteralPath $temporary -Encoding utf8
        Move-Item -LiteralPath $temporary -Destination $Path -Force
    } finally {
        if (Test-Path -LiteralPath $temporary) {
            Remove-Item -LiteralPath $temporary -Force
        }
    }
}

function Assert-NoMatchAnomaly {
    param(
        [Parameter(Mandatory)][string]$LogPath,
        [double]$TimeoutRateCeiling = 0.5
    )

    # ZERO TOLERANCE. A crash, an illegal move, a dropped engine or a protocol
    # error is never normal on this harness: across every stored gate log in
    # tools/results, not one Rarog match has produced any of them. If one
    # appears, the match is describing a broken engine or a broken runner.
    $hard = Select-String -LiteralPath $LogPath `
        -Pattern '(?i)(crashed:\s*[1-9]|disconnect|illegal move|protocol error)' `
        -ErrorAction SilentlyContinue
    if ($hard) {
        throw ("Match contained a crash/illegal-move/disconnect/protocol anomaly " +
               "and is invalid. See '$LogPath'.")
    }

    # RATE-LIMITED. Time forfeits are different: a small background rate is a
    # property of running 14 concurrent games on 14 physical cores, not of the
    # candidate. Measured across the stored logs, healthy matches -- including
    # two null calibrations running the SAME binary on both sides -- sit at
    # 0.03%-0.33%, while the two genuinely poisoned runs sit at 5.52%
    # (p810-mopup, 423/7,665) and 34.85% (VarA-pooled at Threads=4, starved by
    # fastchess 1.8.0 affinity pinning, 184/528). Two orders of magnitude
    # separate them, so a ceiling discriminates and zero tolerance does not.
    #
    # This threshold replaces a zero-tolerance test added in d2c7788 that no
    # match had ever run under. RAR-E06 was the first to reach it: it hit H1
    # at 3,914 games with 3 forfeits (0.077%) and was declared invalid, even
    # though all three flagged sides were already lost by 5-9 pawns and the
    # worst-case reversal of all three moves the estimate ~0.3 Elo against a
    # +22.04 result. Applied to history the old test voided nearly every
    # accepted gate in the project.
    #
    # The count is taken from the per-game 'loses on time' lines rather than
    # the 'Timeouts:' summary, because the summary is per player and would
    # double-count the denominator.
    $forfeits = @(Select-String -LiteralPath $LogPath -Pattern '(?i)loses on time' `
        -ErrorAction SilentlyContinue).Count
    $games = @(Select-String -LiteralPath $LogPath -Pattern '^Finished game \d' `
        -ErrorAction SilentlyContinue).Count
    if ($games -le 0) {
        if ($forfeits -gt 0) {
            throw "Match log records $forfeits time forfeit(s) but no finished games. See '$LogPath'."
        }
        return
    }
    $rate = 100.0 * $forfeits / $games
    if ($rate -gt $TimeoutRateCeiling) {
        # One format string, not a concatenation: `+` binds tighter than `-f`,
        # so "a" + "b" -f $x formats only "b" and throws the placeholders.
        $template = "Time-forfeit rate {0:N3}% ({1}/{2}) exceeds the {3}% ceiling; " +
                    "the match is invalid. See '{4}'."
        throw ($template -f $rate, $forfeits, $games, $TimeoutRateCeiling, $LogPath)
    }
    if ($forfeits -gt 0) {
        Write-Host ("  Time forfeits: {0}/{1} = {2:N3}% (under the {3}% ceiling; recorded, not fatal)" `
            -f $forfeits, $games, $rate, $TimeoutRateCeiling) -ForegroundColor Yellow
    }
}

function Assert-NoAffinityFailure {
    param([Parameter(Mandatory)][string]$LogPath)

    $failure = Select-String -LiteralPath $LogPath `
        -Pattern '(?i)(failed to set cpu affinity|no cores available)' `
        -ErrorAction SilentlyContinue
    if ($failure) {
        throw "fastchess reported an affinity failure; the match is invalid. See '$LogPath'."
    }
}

# ─── Guards shared by every game-playing instrument ────────────────────────
# One implementation each, so the fastchess path (`sprt.ps1`, `spsa.ps1`) and
# the Colosseum path (`colosseum.ps1`) cannot drift into disagreeing about what
# a measurable binary is. Each function keeps the refusal that produced it.

function Get-HarnessBusyProcess {
    # Processes whose presence makes a timed measurement meaningless: another
    # engine, another runner, or a compiler competing for the same cores. The
    # current process is excluded so a wrapper never reports itself.
    #
    # `colosseum-cli` is here; the `colosseum` desktop application is NOT. An
    # open window is not a measurement, and refusing for it would teach the
    # operator to pass -AllowBusyHost out of habit, which is worse than not
    # checking. A tournament the application is actually running shows up as its
    # engine children, which these patterns do catch.
    $patterns = @(
        'rarog*', 'fastchess*', 'colosseum-cli*', 'cutechess*',
        'stockfish*', 'basilisk*', 'reckless*', 'cargo*', 'rustc*'
    )
    $self = $PID
    @(Get-Process -ErrorAction SilentlyContinue | Where-Object {
        if ($_.Id -eq $self) { return $false }
        $name = $_.ProcessName.ToLowerInvariant()
        foreach ($pattern in $patterns) { if ($name -like $pattern) { return $true } }
        $false
    })
}

function Get-HarnessHostBusyPercent {
    # Sustained load of everything except this process, from the raw kernel
    # counters: busy = 1 - idle-ticks / elapsed-ticks over one window. The
    # formatted counter this replaced was read three times right after the
    # wrapper's own start-up (PowerShell's JIT, Get-Process over every process,
    # the CIM provider warming up) and reported that start-up as host load, so a
    # quiet box read 19%. The window opens only after a settle pause, and the
    # wrapper's own CPU over the window is subtracted, so the check measures the
    # host, never itself.
    param([double]$WindowSeconds = 2.0, [double]$SettleSeconds = 1.0)

    if (-not $script:HarnessIsWindows) { return $null }
    $cpus = [Environment]::ProcessorCount
    $read = {
        $raw = Get-CimInstance Win32_PerfRawData_PerfOS_Processor -ErrorAction SilentlyContinue |
            Where-Object { $_.Name -eq '_Total' } | Select-Object -First 1
        if (-not $raw) { return $null }
        [pscustomobject]@{
            Idle = [double]$raw.PercentProcessorTime   # idle time, in 100 ns ticks
            Time = [double]$raw.Timestamp_Sys100NS
            Self = (Get-Process -Id $PID).TotalProcessorTime.TotalSeconds
        }
    }
    Start-Sleep -Milliseconds ([int](1000 * $SettleSeconds))
    $first = & $read
    Start-Sleep -Milliseconds ([int](1000 * $WindowSeconds))
    $second = & $read
    if (-not $first -or -not $second) { return $null }
    $elapsed = $second.Time - $first.Time
    if ($elapsed -le 0) { return $null }
    $busy = 100.0 * (1.0 - ($second.Idle - $first.Idle) / $elapsed)
    # This process's share of the whole machine over the same window.
    $self = 100.0 * ($second.Self - $first.Self) / ($elapsed / 1e7) / $cpus
    [Math]::Max(0.0, $busy - $self)
}

function Assert-HarnessHostIdle {
    # AGENTS.md, "Measurement": measure only on an idle host; if the machine is
    # busy, stop and ask rather than measure. RAR-M48's first pool was discarded
    # for exactly this, and until now the rule lived only in prose — an operator
    # could start a gate on a loaded box and nothing would say so.
    param(
        [double]$MaxBusyPercent = 15,
        [switch]$Allow,
        [switch]$Quiet
    )

    $busy = @(Get-HarnessBusyProcess)
    $percent = Get-HarnessHostBusyPercent
    $reasons = @()
    if ($busy.Count -gt 0) {
        $listed = ($busy | ForEach-Object { "$($_.ProcessName) ($($_.Id))" }) -join ', '
        $reasons += "engine, harness or build processes are running: $listed"
    }
    if ($null -ne $percent -and $percent -gt $MaxBusyPercent) {
        $reasons += ("host CPU is {0:N0}%, over the {1:N0}% ceiling" -f $percent, $MaxBusyPercent)
    }

    $state = [pscustomobject]@{
        BusyPercent   = $percent
        BusyProcesses = @($busy | ForEach-Object { "$($_.ProcessName):$($_.Id)" })
        Reasons       = $reasons
        # A waiver only counts when something was actually waived, so a manifest
        # never claims a busy host that was not there.
        Waived        = ([bool]$Allow -and $reasons.Count -gt 0)
    }

    if ($reasons.Count -eq 0) {
        if (-not $Quiet) {
            Write-Host ("  Host idle: {0} (no engine, harness or build process)" -f
                $(if ($null -eq $percent) { "CPU unreadable" } else { "{0:N0}% CPU" -f $percent }))
        }
        return $state
    }
    if ($Allow) {
        Write-Warning ("HOST NOT IDLE and the check was waived: " + ($reasons -join '; ') +
            ". The result carries this in its manifest and is not comparable with an idle-host run.")
        return $state
    }
    throw ("HOST NOT IDLE - " + ($reasons -join '; ') + ".`n" +
           "A timed game measures the host as much as the engine. Stop the other work and " +
           "re-run, or pass -AllowBusyHost and say why in the registration.")
}

function Assert-AdvertisedOptions {
    # fastchess only WARNS about an option an engine does not expose and then
    # plays the whole match at the DEFAULT — a completed run that measured
    # something else. Refuse instead (PROCESS.md, "Matched ablation", rule 0).
    param([object[]]$Advertised, [string[]]$Wanted, [string]$Label)

    if (-not $Wanted -or @($Wanted).Count -eq 0) { return }
    $normalize = { param($value) ($value -replace '\s+', ' ').Trim().ToLowerInvariant() }
    $have = @($Advertised | ForEach-Object { & $normalize $_.Name })
    $missing = @($Wanted | Where-Object { $_ } |
        ForEach-Object { ($_ -split '=', 2)[0] } |
        Where-Object { $have -notcontains (& $normalize $_) })
    if ($missing.Count -gt 0) {
        throw ("$Label does not advertise: $($missing -join ', '). Rebuild it before measuring; " +
               "the runner would otherwise play the match at default values.")
    }
}

function Assert-EngineProvenance {
    # The sidecar `build_test.ps1` writes beside every test binary is the only
    # thing that binds a measurement to a revision, a compiler and a bench.
    # Every check here exists because its absence once produced a wrong number.
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label,
        [ValidateSet("gate", "tune")][string]$Kind = "gate",
        [switch]$RequireManifest,
        [switch]$RequireBinaryHash,
        [switch]$AllowDirtyTree,
        [string]$ExpectRevision = ""
    )

    $manifestPath = [System.IO.Path]::ChangeExtension($Path, ".json")
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        if ($RequireManifest) {
            throw "Missing engine manifest: $manifestPath. Rebuild with tools/build_test.ps1."
        }
        Write-Host "NOTE: no manifest next to $(Split-Path $Path -Leaf) (pre-9.7 build) — result will lack provenance for $Label." -ForegroundColor Yellow
        return $null
    }

    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    if ($manifest.engine -and $manifest.engine -ne (Split-Path $Path -Leaf)) {
        throw "Manifest for $Label names '$($manifest.engine)', not the selected binary."
    }
    if ($manifest.binary_sha256) {
        $actual = Get-HarnessSha256 $Path
        if ($actual -ne $manifest.binary_sha256) {
            throw ("PROVENANCE MISMATCH - sidecar does not describe the selected binary.`n" +
                   "  Engine:  $Label`n  Actual:  $actual`n" +
                   "  Sidecar: $($manifest.binary_sha256)`nRebuild with tools/build_test.ps1.")
        }
    } elseif ($RequireBinaryHash) {
        throw "Manifest for $Label is not bound to a binary SHA-256; rebuild with tools/build_test.ps1."
    } else {
        Write-Warning "Legacy manifest for $Label is not bound to its binary SHA-256."
    }
    if ($manifest.verification -and $manifest.verification -ne "bench") {
        throw "Manifest for $Label records '$($manifest.verification)', not bench verification."
    }
    if ($Kind -eq "gate" -and $manifest.flavor -like "*-tune") {
        throw "Manifest for $Label is a tune build; rebuild a PGO gate binary."
    }
    if ($Kind -eq "tune" -and $manifest.flavor -notlike "*-tune") {
        throw "A tune requires a tune build manifest; $Label's flavor is '$($manifest.flavor)'."
    }
    # A dirty tree is a REFUSAL, not a warning. AGENTS.md's evidence rule says a
    # ledger row must reproduce its artifact without the branch it came from,
    # and a binary built from uncommitted changes cannot, by construction.
    if ($manifest.git_dirty -and -not $AllowDirtyTree) {
        throw ("DIRTY TREE - $Label was built from uncommitted changes at " +
               "$($manifest.git_sha), so this result cannot be reproduced " +
               "from git alone.`nCommit the change and rebuild with " +
               "tools/build_test.ps1, or pass -AllowDirtyTree and say why in " +
               "the EXPERIMENTS.md registration.")
    }
    if ($manifest.git_dirty) {
        Write-Warning ("$Label was built from a DIRTY tree and -AllowDirtyTree " +
                       "was passed. This result is not reproducible from git.")
    }
    if ($manifest.git_sha -and $ExpectRevision -and $manifest.git_sha -notlike "$ExpectRevision*") {
        throw ("WRONG REVISION - $Label was built at $($manifest.git_sha), " +
               "not the expected $ExpectRevision.`nA gate that measures a " +
               "different revision than the one it registers is not evidence " +
               "for that revision.")
    }
    $manifest
}

function Assert-EngineArmEquality {
    # The compiler-equality guard is the toolchain-pin analogue for BINARIES,
    # and no null pair can see what it catches: a null runs ONE binary against
    # itself, so both sides always share a compiler. The 1.97.0 -> 1.97.1 split
    # of 2026-07-19 folded a per-binary constant into three unrelated gates.
    param(
        [Parameter(Mandatory)][hashtable]$Manifests,
        [Parameter(Mandatory)][string]$LabelA,
        [Parameter(Mandatory)][string]$LabelB,
        [switch]$Quiet
    )

    foreach ($label in @($LabelA, $LabelB)) {
        if (-not $Manifests.ContainsKey($label) -or $null -eq $Manifests[$label]) {
            Write-Warning ("No manifest for $label - compiler equality NOT checkable. " +
                "Rebuild it with tools/build_test.ps1 before trusting a small verdict.")
            return
        }
    }

    $flavorA = $Manifests[$LabelA].flavor
    $flavorB = $Manifests[$LabelB].flavor
    if ($flavorA -and $flavorB -and $flavorA -ne $flavorB) {
        throw ("BUILD FLAVOR MISMATCH - both sides must use the same target/PGO contract.`n" +
               "  $LabelA : $flavorA`n  $LabelB : $flavorB")
    }
    if ($flavorA -and $flavorB -and -not $Quiet) { Write-Host "  Build flavor equality OK: $flavorA" }

    $compilerA = $Manifests[$LabelA].rustc
    $compilerB = $Manifests[$LabelB].rustc
    if ($compilerA -ne $compilerB) {
        throw ("COMPILER MISMATCH - this match would measure the compiler, not the change.`n" +
               "  $LabelA : $compilerA`n  $LabelB : $compilerB`n" +
               "Rebuild BOTH engines with the pinned toolchain (rust-toolchain.toml) " +
               "via tools/build_test.ps1, then re-run.")
    }
    if (-not $Quiet) { Write-Host "  Compiler equality OK: $compilerA" }
}

function Assert-CoreSurfaceArm {
    # The selectivity core's coordinates exist only in a `b2core` build. An
    # off-arm binary would tune a different search under the same names or fail
    # late on a missing option, so the arm is checked before anything is copied.
    param(
        [Parameter(Mandatory)][string[]]$Names,
        [string]$Flavor,
        [Parameter(Mandatory)][string]$ConfigGroup
    )

    $core = @($Names | Where-Object { $_ -like "Core*" })
    if ($core.Count -gt 0 -and $Flavor -notlike "*b2core*") {
        throw ("Config group '$ConfigGroup' names selectivity-core options ($($core[0]) and " +
               "$($core.Count - 1) more), but the tune binary's flavor is '$Flavor'. " +
               "Build it with ./tools/build_test.ps1 -Tune -Features b2core.")
    }
}

function Assert-TuneSurface {
    # A tune surface is only meaningful against the binary that will run it:
    # every coordinate must exist, be a spin, start at the engine's own default,
    # stay inside the engine's range, and still perturb by at least half a unit
    # at the registered horizon — a step-1 integer knob goes dead after ~894
    # iterations and tunes nothing while looking healthy.
    param(
        [Parameter(Mandatory)][object[]]$Advertised,
        [Parameter(Mandatory)][object]$Surface,
        [Parameter(Mandatory)][int]$Iterations,
        [Parameter(Mandatory)][string]$Label,
        [object]$Fixed = $null,
        [double]$Gamma = 0.102
    )

    $normalize = { param($value) ($value -replace '\s+', ' ').Trim().ToLowerInvariant() }
    $advertisedNames = @($Advertised | ForEach-Object { & $normalize $_.Name })
    $tuned = @($Surface.PSObject.Properties.Name)
    if ($tuned.Count -eq 0) { throw "$Label's surface declares no parameters." }

    $missing = @($tuned | Where-Object { $advertisedNames -notcontains (& $normalize $_) })
    if ($missing.Count -gt 0) {
        throw ("$Label does not advertise: $($missing -join ', '). " +
               "SPSA cannot tune an option the selected binary does not expose.")
    }

    foreach ($parameter in $Surface.PSObject.Properties) {
        $declaration = $Advertised | Where-Object {
            (& $normalize $_.Name) -eq (& $normalize $parameter.Name)
        } | Select-Object -First 1
        if ($declaration.Type -ne 'spin') {
            throw "$($parameter.Name) is advertised as '$($declaration.Type)', not a spin option."
        }
        $value = [int64]$parameter.Value.value
        $minimum = [int64]$parameter.Value.min_value
        $maximum = [int64]$parameter.Value.max_value
        $step = [double]$parameter.Value.step
        if ($value -ne [int64]$declaration.Default -or $minimum -lt $declaration.Min -or
            $maximum -gt $declaration.Max -or $minimum -ge $maximum -or
            $value -lt $minimum -or $value -gt $maximum -or $step -le 0) {
            throw ("Invalid SPSA declaration for $($parameter.Name): config value=$value " +
                   "range=[$minimum,$maximum] step=$step; engine default=$($declaration.Default) " +
                   "range=[$($declaration.Min),$($declaration.Max)].")
        }
        $endPerturbation = $step / [Math]::Pow($Iterations, $Gamma)
        if ($endPerturbation -lt 0.5) {
            throw "$($parameter.Name) perturbation rounds to zero before iteration $Iterations (end=$endPerturbation)."
        }
    }

    if ($Fixed) {
        foreach ($option in $Fixed.PSObject.Properties) {
            if ($advertisedNames -notcontains (& $normalize $option.Name)) {
                throw "$Label does not advertise fixed option '$($option.Name)'."
            }
        }
    }

    $tuned
}
function Get-ColosseumPin {
    # One file names the runner: its source revision and the SHA-256 that is
    # actually enforced. Re-pinning to the published `cli-v0.1.0` archive
    # (PLAN B.2.6.3) is an edit to this file and to nothing else.
    param([Parameter(Mandatory)][string]$PinPath)

    if (-not (Test-Path -LiteralPath $PinPath -PathType Leaf)) {
        throw "Colosseum pin not found: $PinPath"
    }
    $pin = Get-Content -LiteralPath $PinPath -Raw | ConvertFrom-Json
    foreach ($field in @('revision', 'sha256')) {
        if (-not $pin.$field) { throw "$PinPath declares no '$field'." }
    }
    if ($pin.sha256 -notmatch '^[0-9a-fA-F]{64}$') {
        throw "$PinPath declares a malformed SHA-256: '$($pin.sha256)'."
    }
    $pin
}

function Assert-ColosseumCli {
    # A harness binary that is not the pinned one is a silent instrument change:
    # the run completes, reports plausible numbers, and the ledger row names a
    # runner that never played the games. Refuse on any hash difference.
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$PinPath,
        [switch]$Quiet
    )

    $pin = Get-ColosseumPin -PinPath $PinPath
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw ("Colosseum CLI not found at '$Path'. Run ./tools/setup_tools.ps1 to stage the " +
               "build pinned at $($pin.revision).")
    }
    $full = (Resolve-Path -LiteralPath $Path).Path
    $sha = Get-HarnessSha256 $full
    if ($sha -ne $pin.sha256) {
        throw ("COLOSSEUM PIN MISMATCH - the staged runner is not the pinned build.`n" +
               "  Staged: $sha`n  Pinned: $($pin.sha256)  (revision $($pin.revision))`n" +
               "Re-stage with ./tools/setup_tools.ps1, or change the pin deliberately in " +
               "$PinPath and say so in the registration.")
    }
    $version = "$(& $full --version 2>&1 | Select-Object -First 1)".Trim()
    if ($pin.version -and $version -ne $pin.version) {
        throw "Colosseum CLI reports '$version' but $PinPath pins '$($pin.version)'."
    }
    if (-not $Quiet) {
        Write-Host "  Runner pinned OK: $version, $($sha.Substring(0, 8))... at $($pin.revision.Substring(0, 7))"
    }
    [pscustomobject]@{
        Path    = $full
        Sha256  = $sha
        Version = $version
        Pin     = $pin
    }
}

function Get-ColosseumRunStatus {
    # The runner's documented machine view of a run directory: its record, the
    # newest valid checkpoint's aggregates and the journal behind them. The
    # progress text is for people and has changed shape between builds.
    param([Parameter(Mandatory)][string]$CliPath, [Parameter(Mandatory)][string]$Dir)

    $text = & $CliPath status $Dir --json 2>$null
    $exit = $LASTEXITCODE
    if ($exit -ne 0) {
        throw "colosseum-cli status exited $exit on $Dir; a run that cannot be read is not accepted."
    }
    try { ($text -join "`n") | ConvertFrom-Json }
    catch { throw "colosseum-cli status on $Dir did not return JSON: $($text -join ' ')" }
}

function Resolve-ColosseumExit {
    # Colosseum's exit codes are per command and carry the verdict (docs/cli):
    # an SPRT's H0 exits 1 and a cap stop 4, so a non-zero exit is not a
    # failure. An outcome is a finished run whose games still have to pass the
    # fault and recount checks; anything else ends the wrapper.
    param([Parameter(Mandatory)][string]$Mode, [Parameter(Mandatory)][int]$ExitCode)

    $outcomes = switch ($Mode) {
        "sprt"      { @{ 0 = "H1 accepted"; 1 = "H0 accepted"; 4 = "cap reached, inconclusive" } }
        "calibrate" { @{ 0 = "pass"; 1 = "fail"; 4 = "inconclusive" } }
        default     { @{ 0 = "completed" } }
    }
    $invalid = if ($Mode -in @("sprt", "calibrate", "spsa")) { 5 } else { 1 }
    if ($outcomes.ContainsKey($ExitCode)) { return [pscustomobject]@{ Kind = "outcome"; Verdict = $outcomes[$ExitCode] } }
    $kind = switch ($ExitCode) {
        $invalid { "invalid" }
        6        { "cancelled" }
        2        { "refused" }
        default  { "error" }
    }
    [pscustomobject]@{ Kind = $kind; Verdict = $null }
}

function Get-ColosseumRunFaults {
    # Fault counts from the status view's checkpoint. A match, SPRT, calibration
    # or tune counts per side, each side's engine faults including its time
    # losses; a tournament counts engine faults without a time split. Anything
    # else is refused: a guard that cannot read its input must not pass the run.
    param([Parameter(Mandatory)][object]$Status)

    $checkpoint = $Status.durable.checkpoint
    if ($null -eq $checkpoint) {
        throw "FAULT COUNTERS UNREADABLE - the run's status carries no checkpoint, so its faults are unknown."
    }
    $faults = $checkpoint.faults
    if ($null -ne $faults) {
        foreach ($field in @('engine_a', 'engine_b', 'time_losses_a', 'time_losses_b', 'infrastructure')) {
            if ($null -eq $faults.$field) {
                throw "FAULT COUNTERS UNREADABLE - the checkpoint's faults lack '$field'."
            }
        }
        $timeLosses = [int]$faults.time_losses_a + [int]$faults.time_losses_b
        $other = ([int]$faults.engine_a - [int]$faults.time_losses_a) +
                 ([int]$faults.engine_b - [int]$faults.time_losses_b) + [int]$faults.infrastructure
        return [pscustomobject]@{ Split = $true; TimeLosses = $timeLosses; Other = $other }
    }
    if ($null -ne $checkpoint.engine_faults) {
        return [pscustomobject]@{ Split = $false; TimeLosses = $null; Other = [int]$checkpoint.engine_faults }
    }
    throw "FAULT COUNTERS UNREADABLE - the checkpoint carries neither per-side faults nor an engine-fault count."
}

function Assert-ColosseumRunFaults {
    # Zero tolerance for a crash, a dropped engine or an illegal move; a rate
    # ceiling for time losses, because a small background rate is a property of
    # running fourteen concurrent games, not of the candidate (RAR-M14, RAR-E06).
    param(
        [Parameter(Mandatory)][object]$Status,
        [Parameter(Mandatory)][int]$ScoredGames,
        [Parameter(Mandatory)][double]$TimeLossRateCeiling,
        [string]$Dir = ""
    )

    $faults = Get-ColosseumRunFaults -Status $Status
    if (-not $faults.Split) {
        if ($faults.Other -gt 0) {
            throw ("The run recorded $($faults.Other) engine fault(s) and this runner mode does not " +
                   "separate time losses from crashes, illegal moves or dropped engines. The result " +
                   "is not accepted until they are read by hand. See $Dir.")
        }
        return $faults
    }
    if ($faults.Other -gt 0) {
        throw ("The run recorded $($faults.Other) non-time engine fault(s) - a crash, an illegal move or a " +
               "dropped engine is never normal on this harness. The result is invalid. See $Dir.")
    }
    if ($faults.TimeLosses -gt 0) {
        if ($ScoredGames -le 0) {
            throw "The run recorded $($faults.TimeLosses) time loss(es) and no scored game. See $Dir."
        }
        $rate = 100.0 * $faults.TimeLosses / $ScoredGames
        if ($rate -gt $TimeLossRateCeiling) {
            throw ("Time-loss rate {0:N3}% ({1}/{2}) exceeds the {3}% ceiling; the run is invalid. See {4}." `
                -f $rate, $faults.TimeLosses, $ScoredGames, $TimeLossRateCeiling, $Dir)
        }
        Write-Host ("  Time losses: {0}/{1} = {2:N3}% (under the {3}% ceiling; recorded, not fatal)" `
            -f $faults.TimeLosses, $ScoredGames, $rate, $TimeLossRateCeiling) -ForegroundColor Yellow
    }
    $faults
}
