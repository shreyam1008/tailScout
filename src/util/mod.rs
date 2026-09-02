//! Small, pure presentation helpers shared by the UI layer.

/// Turn a raw Tailscale OS string into a friendly label.
pub fn os_label(os: &str) -> String {
    match os.to_ascii_lowercase().as_str() {
        "" => "Unknown".to_string(),
        "linux" => "Linux".to_string(),
        "windows" => "Windows".to_string(),
        "macos" | "darwin" => "macOS".to_string(),
        "ios" => "iOS".to_string(),
        "android" => "Android".to_string(),
        "freebsd" => "FreeBSD".to_string(),
        _ => {
            // Capitalize the first character, leave the rest as-is.
            let mut chars = os.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => os.to_string(),
            }
        }
    }
}

/// Format a byte count into a compact human-readable string.
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if bytes == 0 {
        return "0 B".to_string();
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
