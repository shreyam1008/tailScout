//! TailScout library root.
//!
//! The backend (`tailscale`, `util`) is pure and unit-testable. The `ui` and
//! `app` modules are the libadwaita view layer. The binary (`main.rs`) is a
//! thin shell that calls [`app::run`].

#[cfg(target_os = "linux")]
pub mod app;
pub mod settings;
pub mod tailscale;
#[cfg(target_os = "linux")]
pub mod ui;
pub mod util;
