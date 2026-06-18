//! Minimal blocking client for the Tailscale LocalAPI over its unix socket.
//!
//! `tailscaled` runs an HTTP server on a unix socket. Read endpoints like
//! `/localapi/v0/status` are available to the operator user without spawning a
//! process, which is faster than shelling out to the CLI. We implement just
//! enough HTTP/1.1 to issue a GET and read the response body (Content-Length
//! or chunked). For mutating actions we still prefer the CLI (see `cli.rs`).
//!
//! These calls block, so run them off the GTK main thread.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use super::error::{Result, TailscaleError};
use super::model::Status;

/// Candidate socket paths, in order of preference, across distros.
pub const SOCKET_PATHS: [&str; 3] = [
    "/run/tailscale/tailscaled.sock",
    "/var/run/tailscale/tailscaled.sock",
    "/var/run/tailscaled.socket",
];

/// The LocalAPI requires a valid Host header; the daemon accepts this value.
const LOCAL_HOST: &str = "local-tailscaled.sock";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_secs(8);

/// Return the first socket path that exists, if any.
pub fn socket_path() -> Option<&'static str> {
    SOCKET_PATHS.into_iter().find(|p| Path::new(p).exists())
}

/// Fetch and parse status via the LocalAPI socket.
pub fn status() -> Result<Status> {
    let body = get("/localapi/v0/status")?;
    Ok(Status::from_json(&body)?)
}

/// Issue a GET to the given LocalAPI path and return the response body as text.
fn get(path: &str) -> Result<String> {
    let socket = socket_path()
        .ok_or_else(|| TailscaleError::DaemonUnreachable("tailscaled socket not found".into()))?;

    let mut stream = UnixStream::connect(socket)
        .map_err(|e| TailscaleError::DaemonUnreachable(format!("{socket}: {e}")))?;
    stream.set_read_timeout(Some(READ_TIMEOUT)).ok();
    stream.set_write_timeout(Some(CONNECT_TIMEOUT)).ok();

    let request = format!("GET {path} HTTP/1.1\r\nHost: {LOCAL_HOST}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|e| TailscaleError::DaemonUnreachable(e.to_string()))?;

    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| TailscaleError::DaemonUnreachable(e.to_string()))?;

    parse_response(&raw)
}

/// Parse a raw HTTP/1.1 response into its body text.
///
/// Handles the status line (rejecting non-2xx), `Transfer-Encoding: chunked`,
/// and `Content-Length`. Falls back to "everything after the headers" when the
/// connection was simply closed.
pub fn parse_response(raw: &[u8]) -> Result<String> {
    let split = find_subsequence(raw, b"\r\n\r\n")
        .ok_or_else(|| TailscaleError::Parse("malformed HTTP response (no header end)".into()))?;
    let (head, rest) = raw.split_at(split);
    let body_bytes = &rest[4..]; // skip the \r\n\r\n

    let head_text = String::from_utf8_lossy(head);
    let mut lines = head_text.split("\r\n");

    let status_line = lines
        .next()
        .ok_or_else(|| TailscaleError::Parse("empty HTTP response".into()))?;
    let status_code = parse_status_code(status_line)?;
    if !(200..300).contains(&status_code) {
        let body = String::from_utf8_lossy(body_bytes).trim().to_string();
        return Err(TailscaleError::CommandFailed {
            code: Some(status_code as i32),
            message: if body.is_empty() {
                status_line.to_string()
            } else {
                body
            },
        });
    }

    let chunked = lines.any(|line| {
        let lower = line.to_ascii_lowercase();
        lower.starts_with("transfer-encoding:") && lower.contains("chunked")
    });

    let body = if chunked {
        dechunk(body_bytes)?
    } else {
        body_bytes.to_vec()
    };

    String::from_utf8(body).map_err(|e| TailscaleError::Parse(e.to_string()))
}

fn parse_status_code(status_line: &str) -> Result<u16> {
    // e.g. "HTTP/1.1 200 OK"
    status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| TailscaleError::Parse(format!("bad status line: {status_line}")))
}

/// Decode a chunked transfer-encoding body.
fn dechunk(mut data: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    loop {
        let line_end = find_subsequence(data, b"\r\n")
            .ok_or_else(|| TailscaleError::Parse("truncated chunk size".into()))?;
        let size_str = String::from_utf8_lossy(&data[..line_end]);
        // Chunk size may carry extensions after ';'; ignore them.
        let size_hex = size_str.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_hex, 16)
            .map_err(|_| TailscaleError::Parse(format!("bad chunk size: {size_hex}")))?;

        data = &data[line_end + 2..];
        if size == 0 {
            break;
        }
        if data.len() < size {
            return Err(TailscaleError::Parse("truncated chunk body".into()));
        }
        out.extend_from_slice(&data[..size]);
        // Skip chunk data and its trailing CRLF.
        data = &data[size..];
        if data.len() >= 2 {
            data = &data[2..];
        }
    }
    Ok(out)
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
