# TailScout for Windows

Native Windows preview shell for TailScout built with C#, WinUI 3, and the
Windows App SDK. It is intentionally separate from the Linux GTK app so
Windows can use Fluent controls and Windows packaging without bringing GTK to
the platform.

## Requirements

- Windows 10 2004 or later / Windows 11
- .NET 10 SDK
- Visual Studio Build Tools with the Windows App SDK workloads, or a GitHub
  Actions `windows-latest` runner
- Tailscale for Windows installed, with `tailscale.exe` available in `PATH`

Set `TAILSCOUT_TAILSCALE_BIN` to an absolute `tailscale.exe` path if the CLI is
not discoverable from `PATH`.

## Build and Test

```powershell
cd platform\windows
dotnet restore .\TailScout.Windows.sln
dotnet test .\TailScout.Windows.Tests\TailScout.Windows.Tests.csproj -c Release
dotnet build .\TailScout.Windows\TailScout.Windows.csproj -c Release -p:Platform=x64
```

## Publish a ZIP Layout

```powershell
cd platform\windows
dotnet publish .\TailScout.Windows\TailScout.Windows.csproj `
  -c Release `
  -r win-x64 `
  --self-contained true `
  -p:Platform=x64 `
  -p:WindowsPackageType=None `
  -p:WindowsAppSDKSelfContained=true `
  -o ..\..\dist\windows\TailScout
```

The GitHub release workflow zips that output as
`tailscout-v<version>-windows-x64-winui.zip`.

## Measure Working Set

After building or publishing, run the Windows memory helper from PowerShell:

```powershell
cd platform\windows
.\scripts\Measure-TailScoutWindowsMemory.ps1 `
  -ExePath ..\..\dist\windows\TailScout\TailScout.Windows.exe `
  -SettleSeconds 10 `
  -Samples 10 `
  -SampleIntervalMs 1000 `
  -SkipStartupRefresh `
  -BaselineMiB 180
```

If `-ExePath` is omitted, the script looks for the published ZIP layout first,
then the latest local `TailScout.Windows.exe` under `TailScout.Windows\bin`.
It reports `PeakWorkingSet`, `WorkingSet`, and `PrivateMemory` in MiB. When
`-BaselineMiB` is supplied, it exits nonzero if the selected metric exceeds the
baseline; use `-BaselineMetric WorkingSet` or `-BaselineMetric PrivateMemory` to
check a different metric.
`-SkipStartupRefresh` measures the WinUI shell without requiring a running
Tailscale daemon or `tailscale.exe` during the baseline run.

GitHub Actions validates the WinUI build, published layout, and this script's
PowerShell syntax. Run the working-set measurement on an interactive Windows
desktop; hosted Windows runners do not provide a reliable WinUI desktop session
for this measurement.

## Project Layout

- `TailScout.Windows.Core`: pure parser and CLI service library.
- `TailScout.Windows.Tests`: backend parsing and command-contract tests.
- `TailScout.Windows`: unpackaged WinUI 3 desktop app.

## Current Scope

The Windows client uses the Tailscale CLI for both reads and writes:

- `tailscale status --json`
- `tailscale up`, `down`, `login`, `logout`
- `tailscale switch --list --json` and `switch`
- `tailscale set --exit-node=...`
- `tailscale set --advertise-exit-node=true|false`
- `tailscale file cp` and `tailscale file get`
- `tailscale version`, `netcheck`, and `bugreport`

LocalAPI named-pipe support is deliberately not implemented yet; CLI parity is
the first Windows target and needs validation on a real Windows machine.

Cross-platform behavior and UI terminology are defined in
[`../../shared/README.md`](../../shared/README.md).
