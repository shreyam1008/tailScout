//! TailDesk — a clean, native Rust GUI for Tailscale on Linux.

fn main() -> gtk::glib::ExitCode {
    taildesk::app::run()
}
