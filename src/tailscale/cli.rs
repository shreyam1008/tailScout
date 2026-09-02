//! Thin wrappers around the `tailscale` command-line client.
//!
//! The CLI is always version-matched to the running daemon and handles
//! operator authentication, so it is the most robust way to perform mutating
//! actions (up/down, file send). Reads also work here via `status --json`.
//!
//! These functions block, so callers on the GTK thread must run them on a
//! worker thread.

use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use super::error::{Result, TailscaleError};
use super::model::{Profile, Status};

/// Name of the Tailscale binary. Resolved via `$PATH`.
pub const TAILSCALE_BIN: &str = "tailscale";

/// Run the Tailscale CLI with the given args, returning trimmed stdout on
/// success or a descriptive error on failure.
fn run<I, S>(args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_bin(TAILSCALE_BIN, args)
}

fn run_bin<I, S>(bin: &str, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(bin).args(args).output().map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            if bin == TAILSCALE_BIN {
                TailscaleError::CliNotFound
            } else {
                TailscaleError::CommandFailed {
                    code: None,
                    message: format!("Command not found: {bin}"),
                }
            }
        } else {
            TailscaleError::Io(err)
        }
    })?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let message = if !stderr.is_empty() { stderr } else { stdout };

    Err(TailscaleError::CommandFailed {
        code: output.status.code(),
        message,
    })
}

/// Fetch and parse the current daemon status.
pub fn status() -> Result<Status> {
    let json = run(["status", "--json"])?;
    Ok(Status::from_json(&json)?)
}

pub fn profiles() -> Result<Vec<Profile>> {
    let json = run(["switch", "--list", "--json"])?;
    Ok(Profile::parse_list(&json)?)
}

pub fn switch_profile(id_or_name: &str) -> Result<()> {
    run(["switch", id_or_name]).map(|_| ())
}

/// Connect to the tailnet (`tailscale up`).
pub fn up() -> Result<()> {
    run(["up", "--timeout=30s"]).map(|_| ())
}

/// Disconnect from the tailnet (`tailscale down`).
pub fn down() -> Result<()> {
    run(["down"]).map(|_| ())
}

pub fn login() -> Result<String> {
    run(["login", "--timeout=30s"])
}

pub fn logout() -> Result<()> {
    run(["logout"]).map(|_| ())
}

pub fn version() -> Result<String> {
    run(["version"])
}

pub fn netcheck() -> Result<String> {
    run(["netcheck"])
}

pub fn bugreport() -> Result<String> {
    run(["bugreport"])
}

pub fn exit_node_list() -> Result<String> {
    run(["exit-node", "list"])
}

pub fn set_exit_node(target: &str) -> Result<()> {
    let flag = format!("--exit-node={target}");
    run(["set", flag.as_str()]).map(|_| ())
}

pub fn clear_exit_node() -> Result<()> {
    run(["set", "--exit-node="]).map(|_| ())
}

pub fn advertise_exit_node(enabled: bool) -> Result<()> {
    let flag = if enabled {
        "--advertise-exit-node=true"
    } else {
        "--advertise-exit-node=false"
    };
    run(["set", flag]).map(|_| ())
}

pub fn receive_files(directory: &Path) -> Result<()> {
    if !directory.exists() || !directory.is_dir() {
        return Err(TailscaleError::CommandFailed {
            code: None,
            message: format!("not a directory: {}", directory.display()),
        });
    }
    run([
        OsStr::new("file"),
        OsStr::new("get"),
        OsStr::new("--conflict=rename"),
        directory.as_os_str(),
    ])
    .map(|_| ())
}

pub fn set_operator_to_current_user() -> Result<()> {
    let user = env::var("USER").map_err(|_| TailscaleError::CommandFailed {
        code: None,
        message: "Could not detect the current username.".to_string(),
    })?;
    let flag = format!("--operator={user}");
    run_bin(
        "pkexec",
        [
            OsStr::new(TAILSCALE_BIN),
            OsStr::new("set"),
            OsStr::new(&flag),
        ],
    )
    .map(|_| ())
}

/// Send a single file to a peer via Taildrop (`tailscale file cp <file> <target>:`).
///
/// `target` should be a Tailscale IP or MagicDNS name. The trailing colon is
/// added here so callers pass a bare address.
pub fn send_file(path: &Path, target: &str) -> Result<()> {
    if !path.exists() {
        return Err(TailscaleError::CommandFailed {
            code: None,
            message: format!("file no longer exists: {}", path.display()),
        });
    }
    if path.is_dir() {
        return Err(TailscaleError::CommandFailed {
            code: None,
            message: "folders are not supported by Taildrop".to_string(),
        });
    }

    let destination = format!("{target}:");
    run([
        OsStr::new("file"),
        OsStr::new("cp"),
        path.as_os_str(),
        OsStr::new(&destination),
    ])
    .map(|_| ())
}
