//! Application wiring: builds the libadwaita `Application` and shows the window.

use adw::prelude::*;
use gtk::glib;

/// Reverse-DNS application identifier (used for D-Bus, icons, settings).
const APP_ID: &str = "dev.shre.TailScout";

/// Build and run the TailScout application. Returns the process exit code.
pub fn run() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| crate::ui::load_css());
    app.connect_activate(|app| {
        let window = crate::ui::build_window(app);
        window.present();
    });

    app.run()
}
