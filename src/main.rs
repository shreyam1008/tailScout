//! TailScout — a clean, native Rust GUI for Tailscale on Linux.

#[cfg(target_os = "linux")]
fn main() -> gtk::glib::ExitCode {
    tailscout::app::run()
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!(
        "The Rust GTK TailScout binary is Linux-only. Use platform/windows or platform/macos for native clients on this OS."
    );
    std::process::exit(1);
}
