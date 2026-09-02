# TailScout contributor rules

Read `README.md` for the product overview and `shared/README.md` for the
cross-platform behavior and UI contract.

## Product shape

TailScout has three native interfaces over the installed Tailscale client:

- Linux: Rust with GTK4/libadwaita (primary, verified client)
- Windows: C# with WinUI 3
- macOS: Swift with SwiftUI

Keep the native interfaces small. Do not introduce Electron, a web view, a heavy
async runtime, a background service, or cross-language FFI without a concrete need.

## Architecture

- Business rules, parsing, validation, and process execution belong in the pure
  platform cores: `src/tailscale/`, `TailScout.Windows.Core`, and `TailScoutCore`.
- UI code renders state, collects input, and invokes its core asynchronously.
- Linux status reads prefer LocalAPI and fall back to the CLI. Mutations use the CLI.
- Windows and macOS currently use the CLI for both reads and mutations.
- Shared behavior and canonical parser inputs live in `shared/`. Every platform must
  test a wire-format change against the same fixture.
- Keep one concern per file and prefer direct code over speculative abstractions.

## Toolchains

| Target | Toolchain | UI |
| --- | --- | --- |
| Linux | stable Rust; MSRV 1.80; edition 2021 | gtk4-rs 0.9, libadwaita 0.7 |
| Windows | .NET 10 | WinUI 3 / Windows App SDK |
| macOS | Swift 6, macOS 13+ | SwiftUI |

Blocking subprocess and socket calls must never run on a UI thread/main actor.

## Repository map

```text
src/tailscale/                 Rust backend
src/ui/window.rs               Linux window coordinator
src/ui/window/                 Focused Linux UI concerns
platform/windows/
  TailScout.Windows.Core/      Typed parser and CLI/process adapter
  TailScout.Windows/           WinUI shell, split by concern
  TailScout.Windows.Tests/     Core and command-contract tests
platform/macos/
  Sources/TailScoutCore/       Typed parser and CLI/process adapter
  Sources/TailScout/           SwiftUI shell and view model
  Tests/TailScoutTests/        Core tests
shared/                        Cross-platform contract and fixtures
tests/                         Rust backend tests
```

## Coding rules

- Handle daemon-down, no-peer, offline-peer, missing-CLI, permission, malformed-data,
  and cancelled-picker paths explicitly.
- Runtime/IO Rust paths return `Result`; no `unwrap()` or `expect()` outside tests.
- Use named constants for protocol paths, binary names, and timeouts.
- Preserve unknown daemon states and tolerate missing or `null` JSON fields.
- Do not duplicate ownership, Taildrop, sorting, or display fallback policy in UI code.
- Keep common section order, terminology, device-row facts, and action availability
  aligned with the UI contract; native controls may differ, product semantics may not.
- `Cargo.toml` is the release-version source of truth. Keep checked metadata aligned
  and run `scripts/check-release-truth.py`.
- Update `CHANGELOG.md` for every user-visible change and whenever the verified
  Tailscale version changes.

## Required checks

```text
cargo fmt --all -- --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
dotnet test platform/windows/TailScout.Windows.Tests/TailScout.Windows.Tests.csproj -c Release
dotnet build platform/windows/TailScout.Windows/TailScout.Windows.csproj -c Release -p:Platform=x64
(cd platform/macos && swift test && swift build -c release)
python scripts/check-release-truth.py
```

Linux GUI changes also need a Linux build and manual launch. Windows/macOS UI changes
need a native build and launch on their respective platforms.
