//! Error type for all Tailscale backend operations.

use std::fmt;

/// Errors that can occur talking to the Tailscale daemon or CLI.
#[derive(Debug)]
pub enum TailscaleError {
    /// The `tailscale` binary could not be found or executed.
    CliNotFound,
    /// The CLI ran but returned a non-zero exit code. Carries stderr/stdout.
    CommandFailed { code: Option<i32>, message: String },
    /// Failed to spawn or wait on the CLI process.
    Io(std::io::Error),
    /// Failed to parse JSON output from the daemon.
    Parse(String),
    /// The daemon is not reachable (socket missing / not running).
    DaemonUnreachable(String),
}

impl fmt::Display for TailscaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CliNotFound => write!(
                f,
                "The 'tailscale' command was not found. Is Tailscale installed and in your PATH?"
            ),
            Self::CommandFailed { code, message } => {
                let suffix = code.map(|c| format!(" (exit {c})")).unwrap_or_default();
                write!(f, "Tailscale command failed{suffix}: {message}")
            }
            Self::Io(err) => write!(f, "I/O error running Tailscale: {err}"),
            Self::Parse(msg) => write!(f, "Could not parse Tailscale output: {msg}"),
            Self::DaemonUnreachable(msg) => {
                write!(f, "Tailscale daemon is not reachable: {msg}")
            }
        }
    }
}

impl std::error::Error for TailscaleError {}

impl TailscaleError {
    pub fn is_permission_problem(&self) -> bool {
        let Self::CommandFailed { message, .. } = self else {
            return false;
        };
        let message = message.to_lowercase();
        [
            "permission denied",
            "access denied",
            "must be root",
            "needs sudo",
            "sudo tailscale",
            "operator",
            "not permitted",
        ]
        .iter()
        .any(|needle| message.contains(needle))
    }
}

impl From<std::io::Error> for TailscaleError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::CliNotFound,
            _ => Self::Io(err),
        }
    }
}

impl From<serde_json::Error> for TailscaleError {
    fn from(err: serde_json::Error) -> Self {
        Self::Parse(err.to_string())
    }
}

/// Convenience alias for backend results.
pub type Result<T> = std::result::Result<T, TailscaleError>;
