//! The view layer: a thin libadwaita UI over the Tailscale backend.

mod device_row;
mod dialogs;
mod help;
mod window;

pub use window::build_window;

use gtk::gdk::Display;
use gtk::CssProvider;

/// Load TailScout's custom CSS into the default display.
pub fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_string(include_str!("style.css"));
    if let Some(display) = Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
