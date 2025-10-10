//! TailScout — a clean, native Rust GUI for Tailscale on Linux.

fn main() -> gtk::glib::ExitCode {
    tailscout::app::run()
}
