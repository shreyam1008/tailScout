# Runtime Footprint

TailScout is native on each desktop. The baseline targets are tuned per platform:
GTK/libadwaita on Linux, WinUI 3 on Windows, and SwiftUI on macOS.

Memory metrics are intentionally platform-specific:

- **Linux:** use PSS because RSS also includes shared toolkit/framework pages already loaded by the desktop session.
- **Windows:** use process working set/private memory. WinUI has a higher process floor than pure app code.
- **macOS:** use RSS from `ps`; SwiftUI/AppKit frameworks are mostly system-supplied and shared, while the application process remains small.

## Baselines

These are smoke-test ceilings, not strict budgets. If a platform exceeds its
ceiling, fix the regression before changing the value.

| Platform | Metric | CI/Smoke target | Recent observed |
|---|---|---:|---:|
| Linux GTK/libadwaita (desktop) | PSS | `100 MiB` | `68.8 MiB` (Ubuntu GNOME, release build, idle) |
| Linux GTK/libadwaita (GitHub Actions xvfb) | PSS | `260 MiB` | `232.1 MiB` peak |
| Windows WinUI 3 | Peak working set | `180 MiB` | not measured on hosted GitHub runners |
| macOS SwiftUI | RSS | `120 MiB` | `82.9 MiB` |

## Commands

Run each check from the relevant directory with release artifacts available.

Linux:

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

Windows (interactive desktop recommended):

```powershell
cd platform\windows
.\scripts\Measure-TailScoutWindowsMemory.ps1 `
  -ExePath ..\..\dist\windows\TailScout\TailScout.Windows.exe `
  -SkipStartupRefresh `
  -BaselineMiB 180
```

GitHub Actions validates the WinUI build, publish layout, and measurement script
syntax. Run this command on an interactive Windows desktop to measure the real
working-set baseline.

macOS:

```bash
cd platform/macos
Scripts/measure_rss.sh --build release --baseline-mib 120
```

## How to update baselines

- Update target values in the matrix/CLI arguments used by CI and local scripts after
  confirming new usage is from intentional feature work.
- Keep release checks using release artifacts:
  - `TAILSCOUT_MEMORY_LIMIT_MIB=... scripts/measure-memory.sh`
  - `Measure-TailScoutWindowsMemory.ps1 -BaselineMiB ...`
  - `Scripts/measure_rss.sh --baseline-mib ...`

If a baseline rises after an intentional feature set, call that out explicitly in
`CHANGELOG.md`.

## Rules

- Run memory tests against release builds.
- Keep TailScout closed before starting a measurement.
- Do not compare Linux RSS to Windows working set directly; the metrics count
  shared memory differently.
- A memory regression should be fixed before changing the baseline unless the
  feature intentionally adds resident state.
