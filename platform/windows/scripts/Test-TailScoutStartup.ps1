param([Parameter(Mandatory)][string]$Executable)
$ErrorActionPreference = 'Stop'
$executablePath = (Resolve-Path -LiteralPath $Executable).Path
$start = [Diagnostics.ProcessStartInfo]::new($executablePath, '--startup-check')
$start.UseShellExecute = $false
$start.CreateNoWindow = $true
$start.RedirectStandardOutput = $true
$start.RedirectStandardError = $true
$process = [Diagnostics.Process]::Start($start)
try {
    $stdout = $process.StandardOutput.ReadToEndAsync()
    $stderr = $process.StandardError.ReadToEndAsync()
    if (-not $process.WaitForExit(30000)) {
        $process.Kill($true)
        throw 'Published WinUI startup check timed out.'
    }
    $output = $stdout.GetAwaiter().GetResult()
    $errorOutput = $stderr.GetAwaiter().GetResult()
    if ($process.ExitCode -ne 0 -or $output -notmatch 'TailScout startup check passed') {
        throw "Published WinUI startup failed (exit $($process.ExitCode)): $errorOutput $output"
    }
    Write-Output $output.Trim()
} finally { $process.Dispose() }
