//! TailScout library root.
//!
//! The backend (`tailscale`, `util`) is pure and unit-testable. The `ui` and
//! `app` modules are the libadwaita view layer. The binary (`main.rs`) is a
//! thin shell that calls [`app::run`].

pub mod app;
pub mod settings;
pub mod tailscale;
pub mod ui;
pub mod util;
