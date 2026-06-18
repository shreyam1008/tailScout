//! Tailscale backend: typed models and clients for the daemon.
//!
//! This module is pure (no GTK) and unit-testable. The UI layer talks only to
//! the small facade exposed here.

pub mod cli;
pub mod error;
#[cfg(unix)]
pub mod localapi;
pub mod model;

pub use error::{Result, TailscaleError};
pub use model::{BackendState, Node, Profile, Status};

use std::path::Path;

/// Fetch the current status, preferring the fast LocalAPI socket and falling
/// back to the CLI if the socket is unavailable or denied.
pub fn fetch_status() -> Result<Status> {
    #[cfg(unix)]
    {
        match localapi::status() {
            Ok(status) => Ok(status),
            Err(_) => cli::status(),
        }
    }

    #[cfg(not(unix))]
    {
        cli::status()
    }
}

/// Connect to the tailnet.
pub fn connect() -> Result<()> {
    cli::up()
}

/// Disconnect from the tailnet.
pub fn disconnect() -> Result<()> {
    cli::down()
}

pub fn login() -> Result<String> {
    cli::login()
}

pub fn logout() -> Result<()> {
    cli::logout()
}

pub fn profiles() -> Result<Vec<Profile>> {
    cli::profiles()
}

pub fn switch_profile(id_or_name: &str) -> Result<()> {
    cli::switch_profile(id_or_name)
}

pub fn set_exit_node(target: &str) -> Result<()> {
    cli::set_exit_node(target)
}

pub fn clear_exit_node() -> Result<()> {
    cli::clear_exit_node()
}

pub fn advertise_exit_node(enabled: bool) -> Result<()> {
    cli::advertise_exit_node(enabled)
}

pub fn version() -> Result<String> {
    cli::version()
}

pub fn netcheck() -> Result<String> {
    cli::netcheck()
}

pub fn bugreport() -> Result<String> {
    cli::bugreport()
}

pub fn exit_node_list() -> Result<String> {
    cli::exit_node_list()
}

pub fn receive_files(directory: &Path) -> Result<()> {
    cli::receive_files(directory)
}

pub fn set_operator_to_current_user() -> Result<()> {
    cli::set_operator_to_current_user()
}

/// Send a file to a peer via Taildrop.
pub fn send_file(path: &Path, target: &str) -> Result<()> {
    cli::send_file(path, target)
}
