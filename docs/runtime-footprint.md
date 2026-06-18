# Runtime Footprint

TailScout is native on each desktop. The app should stay close to the platform
toolkit floor: GTK/libadwaita on Linux, WinUI 3 on Windows, and SwiftUI on macOS.

Memory metrics are platform-specific:

- **Linux:** use PSS as the release gate because RSS includes shared GTK,
  libadwaita, graphics, font, and theme pages already loaded by the desktop.
- **Windows:** use working set/private memory from the process. WinUI and the
  Windows App SDK have a higher floor than the app code itself.
- **macOS:** use RSS from `ps`. SwiftUI/AppKit frameworks are mostly system
  supplied, so the package is small even though the process still maps system
  frameworks at runtime.

## Baselines

These are smoke-test ceilings, not target allocations. If a platform crosses
the ceiling, investigate before raising it.

| Platform | Metric | Current baseline | Notes |
|---|---:|---:|---|
| Linux GTK/libadwaita desktop | PSS | 100 MiB | Measured locally at ~68 MiB PSS on Ubuntu GNOME |
| Linux GTK/libadwaita CI headless | PSS | 260 MiB | GitHub `xvfb` runner measured ~225 MiB PSS because toolkit/graphics pages are not shared with a normal desktop session |
| Windows WinUI 3 desktop | Peak working set | 180 MiB | Run locally on an interactive Windows desktop; GitHub hosted Windows runners build/package but cannot provide a reliable WinUI desktop session for memory sampling |
| macOS SwiftUI | Peak RSS | 120 MiB | GitHub macOS runner measured ~83 MiB RSS |

## Commands

Linux:

```bash
TAILSCOUT_MEMORY_LIMIT_MIB=100 scripts/measure-memory.sh
```

Windows:

```powershell
cd platform\windows
.\scripts\Measure-TailScoutWindowsMemory.ps1 `
  -ExePath ..\..\dist\windows\TailScout\TailScout.Windows.exe `
  -SkipStartupRefresh `
  -BaselineMiB 180
```

GitHub Actions validates the Windows build, publish layout, and measurement
script syntax. Run the command above on a Windows desktop to enforce the actual
WinUI working-set baseline.

macOS:

```bash
cd platform/macos
Scripts/measure_rss.sh --build release --baseline-mib 120
```

## Rules

- Run memory tests against release builds.
- Keep TailScout closed before starting a measurement.
- Do not compare Linux RSS to Windows working set directly; the metrics count
  shared memory differently.
- A memory regression should be fixed before changing the baseline unless the
  feature intentionally adds resident state.
