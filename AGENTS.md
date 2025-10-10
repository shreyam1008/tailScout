# TailDesk — Agent Rules

Rules for AI coding agents and human contributors working on TailDesk.
Read `README.md` for the project overview.

## What this is

A **native Rust GUI for Tailscale**, built with **GTK4 + libadwaita**. Linux first;
Windows and macOS planned. The goal is a clean, low-RAM, modern desktop client that
matches or beats Trayscale, and grows as Tailscale adds features.

## Core principles

- **Minimal & clean.** Prefer the simplest correct solution. No speculative abstractions.
- **Native look.** Use libadwaita widgets and patterns. Do not hand-roll custom chrome.
- **Backend stays pure.** All Tailscale logic lives in `src/tailscale/` and must be
  unit-testable without a GUI. The `src/ui/` layer is a thin view.
- **Version-aware.** TailDesk reads the daemon version at runtime. When Tailscale adds
  features, extend the backend, bump the verified version in `CHANGELOG.md`.
- **Edge cases first.** Daemon down, no peers, offline peers, missing CLI, permission
  denied — handle them explicitly and surface friendly messages.

## Stack & versions

| Tool | Version | Notes |
|---|---|---|
| Rust | stable (1.96+) | edition 2021 |
| gtk4-rs | 0.9 | `gtk4` crate, feature `v4_12` |
| libadwaita | 0.7 | `libadwaita` crate, feature `v1_5` |
| serde / serde_json | 1 | parsing `tailscale status --json` |

Do not add heavy async runtimes. Blocking CLI calls run on worker threads and post
results back to the GTK main loop via channels.

## File structure

```
src/
├── main.rs            — entry: init adw, build Application, run
├── app.rs             — Application wiring and top-level state
├── tailscale/
│   ├── mod.rs         — TailscaleClient facade (re-exports)
│   ├── model.rs       — typed data models (serde)
│   ├── error.rs       — error type
│   ├── cli.rs         — `tailscale` CLI wrappers (actions + status)
│   └── localapi.rs    — LocalAPI unix-socket client (reads)
├── ui/
│   ├── mod.rs
│   ├── window.rs      — main window layout
│   ├── device_row.rs  — device list row widget
│   └── style.css      — custom CSS
└── util/
    └── mod.rs

tests/
└── parsing.rs         — model/parsing tests
```

### Rules
- One concern per file. Keep `main.rs` thin.
- New Tailscale capability = add to `tailscale/` (model + cli/localapi), then surface in `ui/`.
- Never block the GTK main thread on a subprocess or socket. Use threads + channels.
- Every backend parsing/transform function gets a test in `tests/`.

## Conventions

- `cargo fmt` and `cargo clippy` must pass clean before any commit.
- No `unwrap()`/`expect()` on runtime/IO paths; return `Result` and show errors in the UI.
  `unwrap()` is acceptable in tests.
- Named constants over magic strings (socket path, CLI name, refresh interval).
- Update `CHANGELOG.md` for every user-visible change, including the verified Tailscale version.

## Testing

- Backend logic: unit/integration tests in `tests/` (no GUI needed).
- Run `cargo test` before confirming any change.
- GUI changes: at minimum confirm `cargo build` and a manual launch.
