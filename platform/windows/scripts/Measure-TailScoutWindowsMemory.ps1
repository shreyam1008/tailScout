[CmdletBinding()]
param(
    [string]$ExePath,

    [string[]]$AppArgumentList = @(),

    [ValidateRange(0, 300)]
    [int]$IdleTimeoutSeconds = 15,

    [ValidateRange(0, 300)]
    [int]$SettleSeconds = 8,

    [ValidateRange(1, 1000)]
    [int]$Samples = 10,

    [ValidateRange(100, 600000)]
    [int]$SampleIntervalMs = 1000,

    [double]$BaselineMiB = 0,

    [ValidateSet("PeakWorkingSet", "WorkingSet", "PrivateMemory")]
    [string]$BaselineMetric = "PeakWorkingSet",

    [switch]$LeaveRunning,

    [switch]$Json
)

Set-StrictMode -Version 2.0
$ErrorActionPreference = "Stop"

function ConvertTo-MiB {
    param(
        [long]$Bytes
    )

    [Math]::Round($Bytes / 1MB, 2)
}

function Resolve-TailScoutExe {
    param(
        [string]$Path
    )

    if ($Path) {
        $resolvedPath = Resolve-Path -LiteralPath $Path
        return $resolvedPath.ProviderPath
    }

    $scriptRoot = $PSScriptRoot
    $windowsRoot = Split-Path -Parent $scriptRoot
    $repoRoot = Split-Path -Parent (Split-Path -Parent $windowsRoot)

    $candidatePaths = @(
        (Join-Path $repoRoot "dist\windows\TailScout\TailScout.Windows.exe")
    )

    $buildRoots = @(
        (Join-Path $windowsRoot "TailScout.Windows\bin\Release"),
        (Join-Path $windowsRoot "TailScout.Windows\bin\Debug")
    )

    foreach ($buildRoot in $buildRoots) {
        if (-not (Test-Path -LiteralPath $buildRoot)) {
            continue
        }

        $buildMatches = @(
            Get-ChildItem `
                -LiteralPath $buildRoot `
                -Filter "TailScout.Windows.exe" `
                -Recurse `
                -File `
                -ErrorAction SilentlyContinue |
                Sort-Object LastWriteTime -Descending
        )

        foreach ($match in $buildMatches) {
            $candidatePaths += $match.FullName
        }
    }

    foreach ($candidatePath in $candidatePaths) {
        if (Test-Path -LiteralPath $candidatePath -PathType Leaf) {
            $resolvedPath = Resolve-Path -LiteralPath $candidatePath
            return $resolvedPath.ProviderPath
        }
    }

    throw "TailScout.Windows.exe was not found. Build or publish first, or pass -ExePath with the executable path."
}

function Wait-ForInputIdle {
    param(
        [System.Diagnostics.Process]$Process,
        [int]$TimeoutSeconds
    )

    if ($TimeoutSeconds -le 0) {
        return "skipped"
    }

    try {
        if ($Process.WaitForInputIdle($TimeoutSeconds * 1000)) {
            return "ready"
        }

        return "timed out after $TimeoutSeconds second(s)"
    }
    catch {
        return "not available: $($_.Exception.Message)"
    }
}

function Get-MetricValue {
    param(
        [pscustomobject]$Summary,
        [string]$MetricName
    )

    switch ($MetricName) {
        "PeakWorkingSet" { return $Summary.PeakWorkingSetMiB }
        "WorkingSet" { return $Summary.WorkingSetMiB }
        "PrivateMemory" { return $Summary.PrivateMemoryMiB }
    }
}

function Get-Maximum {
    param(
        [object[]]$Rows,
        [string]$PropertyName
    )

    ($Rows | Measure-Object -Property $PropertyName -Maximum).Maximum
}

$hasBaseline = $PSBoundParameters.ContainsKey("BaselineMiB")
$process = $null
$exitCode = 0

try {
    if ($hasBaseline -and $BaselineMiB -le 0) {
        throw "-BaselineMiB must be greater than zero when supplied."
    }

    $resolvedExePath = Resolve-TailScoutExe -Path $ExePath

    if (-not $Json) {
        Write-Host "Launching TailScout.Windows.exe"
        Write-Host ("Executable: {0}" -f $resolvedExePath)
    }

    $startProcessArgs = @{
        FilePath = $resolvedExePath
        PassThru = $true
    }

    if ($null -ne $AppArgumentList -and $AppArgumentList.Count -gt 0) {
        $startProcessArgs.ArgumentList = $AppArgumentList
    }

    $process = Start-Process @startProcessArgs

    $idleStatus = Wait-ForInputIdle -Process $process -TimeoutSeconds $IdleTimeoutSeconds

    if ($process.HasExited) {
        throw "TailScout.Windows.exe exited before memory sampling could start. Exit code: $($process.ExitCode)"
    }

    if ($SettleSeconds -gt 0) {
        Start-Sleep -Seconds $SettleSeconds
    }

    $sampleRows = @()

    for ($index = 1; $index -le $Samples; $index++) {
        try {
            $liveProcess = [System.Diagnostics.Process]::GetProcessById($process.Id)
        }
        catch {
            throw "TailScout.Windows.exe exited before sample $index could be collected."
        }

        $liveProcess.Refresh()

        $sampleRows += [pscustomobject]@{
            Sample               = $index
            Timestamp            = (Get-Date).ToString("o")
            PeakWorkingSetMiB    = ConvertTo-MiB -Bytes $liveProcess.PeakWorkingSet64
            WorkingSetMiB        = ConvertTo-MiB -Bytes $liveProcess.WorkingSet64
            PrivateMemoryMiB     = ConvertTo-MiB -Bytes $liveProcess.PrivateMemorySize64
        }

        if ($index -lt $Samples) {
            Start-Sleep -Milliseconds $SampleIntervalMs
        }
    }

    $summary = [pscustomobject]@{
        Executable           = $resolvedExePath
        ProcessId            = $process.Id
        IdleStatus           = $idleStatus
        SettleSeconds        = $SettleSeconds
        Samples              = $Samples
        SampleIntervalMs     = $SampleIntervalMs
        PeakWorkingSetMiB    = Get-Maximum -Rows $sampleRows -PropertyName "PeakWorkingSetMiB"
        WorkingSetMiB        = Get-Maximum -Rows $sampleRows -PropertyName "WorkingSetMiB"
        PrivateMemoryMiB     = Get-Maximum -Rows $sampleRows -PropertyName "PrivateMemoryMiB"
        BaselineMetric       = if ($hasBaseline) { $BaselineMetric } else { $null }
        BaselineMiB          = if ($hasBaseline) { [Math]::Round($BaselineMiB, 2) } else { $null }
        BaselineExceeded     = $false
    }

    if ($hasBaseline) {
        $actualMiB = Get-MetricValue -Summary $summary -MetricName $BaselineMetric
        $summary.BaselineExceeded = $actualMiB -gt $BaselineMiB

        if ($summary.BaselineExceeded) {
            $exitCode = 2
        }
    }

    if ($Json) {
        [pscustomobject]@{
            Summary = $summary
            Samples = $sampleRows
        } | ConvertTo-Json -Depth 4
    }
    else {
        Write-Host ""
        Write-Host "Samples (MiB)"
        $sampleRows |
            Format-Table `
                Sample,
                PeakWorkingSetMiB,
                WorkingSetMiB,
                PrivateMemoryMiB `
                -AutoSize

        Write-Host ""
        Write-Host "Summary (maximum sampled MiB)"
        $summary |
            Format-List `
                ProcessId,
                IdleStatus,
                SettleSeconds,
                Samples,
                SampleIntervalMs,
                PeakWorkingSetMiB,
                WorkingSetMiB,
                PrivateMemoryMiB,
                BaselineMetric,
                BaselineMiB,
                BaselineExceeded

        if ($hasBaseline) {
            $actualMiB = Get-MetricValue -Summary $summary -MetricName $BaselineMetric

            if ($summary.BaselineExceeded) {
                Write-Warning ("Baseline exceeded: {0} {1} MiB > {2} MiB" -f $BaselineMetric, $actualMiB, $BaselineMiB)
            }
            else {
                Write-Host ("Baseline met: {0} {1} MiB <= {2} MiB" -f $BaselineMetric, $actualMiB, $BaselineMiB)
            }
        }
    }
}
catch {
    [Console]::Error.WriteLine("Error: {0}" -f $_.Exception.Message)
    $exitCode = 1
}
finally {
    if ($null -ne $process -and -not $LeaveRunning) {
        try {
            if (-not $process.HasExited) {
                Stop-Process -Id $process.Id -ErrorAction SilentlyContinue
            }
        }
        catch {
        }
    }
}

exit $exitCode
