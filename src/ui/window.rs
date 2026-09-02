//! Main GTK window coordinator.
//!
//! Layout, backend actions, accounts, devices, Taildrop, exit nodes, and the
//! overview live in focused submodules. This file only owns shared UI state,
//! event wiring, and status-to-view projection.

mod accounts;
mod actions;
mod devices;
mod exit_nodes;
mod layout;
mod overview;
mod taildrop;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;

use crate::settings::Settings;
use crate::tailscale::{BackendState, Status};
use crate::ui::dialogs;
use crate::util::os_label;

use accounts::show_profiles;
use actions::{
    refresh, set_state_pill, setup_help_menu, setup_operator, show_hint, toggle_connection,
    update_connect_button,
};
use devices::apply_device_list;
use exit_nodes::{
    apply_exit_node_status, apply_local_capabilities, show_advertise_exit_node_dialog,
    show_exit_node_dialog,
};
use overview::show_overview;
use taildrop::{receive_into_default_or_pick, show_preferences, update_taildrop_folder_row};

const WINDOW_WIDTH: i32 = 920;
const WINDOW_HEIGHT: i32 = 700;
const ADMIN_CONSOLE_URL: &str = "https://login.tailscale.com/admin/machines";

struct Ui {
    window: adw::ApplicationWindow,
    toasts: adw::ToastOverlay,
    banner: adw::Banner,
    title: adw::WindowTitle,
    connect_button: gtk::Button,
    refresh_button: gtk::Button,
    spinner: gtk::Spinner,
    setup_button: gtk::Button,
    admin_button: gtk::Button,
    receive_button: gtk::Button,
    profiles_button: gtk::Button,
    help_button: gtk::MenuButton,
    self_row: adw::ActionRow,
    state_label: gtk::Label,
    taildrop_folder_row: adw::ActionRow,
    exit_row: adw::ActionRow,
    advertise_exit_row: adw::ActionRow,
    devices_group: adw::PreferencesGroup,
    device_list: gtk::ListBox,
    busy: Cell<bool>,
    file_dialog_open: Cell<bool>,
    backend_state: RefCell<BackendState>,
    last_status: RefCell<Option<Status>>,
    settings: RefCell<Settings>,
    admin_row: adw::ActionRow,
    taildrop_row: adw::ActionRow,
}

pub fn build_window(app: &adw::Application) -> adw::ApplicationWindow {
    let ui = layout::build(app);
    update_taildrop_folder_row(&ui);

    {
        let ui = Rc::clone(&ui);
        ui.refresh_button
            .clone()
            .connect_clicked(move |_| refresh(&ui));
    }
    {
        let ui = Rc::clone(&ui);
        ui.connect_button
            .clone()
            .connect_clicked(move |_| toggle_connection(&ui));
    }
    {
        let ui = Rc::clone(&ui);
        ui.setup_button
            .clone()
            .connect_clicked(move |_| setup_operator(&ui));
    }
    {
        let ui = Rc::clone(&ui);
        ui.admin_button
            .clone()
            .connect_clicked(move |_| dialogs::open_uri(&ui.window, ADMIN_CONSOLE_URL));
    }
    {
        let ui = Rc::clone(&ui);
        ui.admin_row
            .clone()
            .connect_activated(move |_| dialogs::open_uri(&ui.window, ADMIN_CONSOLE_URL));
    }
    {
        let ui = Rc::clone(&ui);
        ui.self_row
            .clone()
            .connect_activated(move |_| show_overview(&ui));
    }
    {
        let ui = Rc::clone(&ui);
        ui.receive_button
            .clone()
            .connect_clicked(move |_| receive_into_default_or_pick(&ui));
    }
    {
        let ui = Rc::clone(&ui);
        ui.taildrop_row
            .clone()
            .connect_activated(move |_| receive_into_default_or_pick(&ui));
    }
    {
        let ui = Rc::clone(&ui);
        ui.taildrop_folder_row
            .clone()
            .connect_activated(move |_| show_preferences(&ui));
    }
    {
        let ui = Rc::clone(&ui);
        ui.profiles_button
            .clone()
            .connect_clicked(move |_| show_profiles(&ui));
    }
    {
        let ui = Rc::clone(&ui);
        ui.exit_row
            .clone()
            .connect_activated(move |_| show_exit_node_dialog(&ui));
    }
    {
        let ui = Rc::clone(&ui);
        ui.advertise_exit_row
            .clone()
            .connect_activated(move |_| show_advertise_exit_node_dialog(&ui));
    }

    setup_help_menu(&ui);
    refresh(&ui);
    ui.window.clone()
}

fn apply_status(ui: &Rc<Ui>, status: &Status) {
    ui.banner.set_revealed(false);
    ui.last_status.replace(Some(status.clone()));

    let running = status.backend_state.is_running();
    ui.backend_state.replace(status.backend_state.clone());
    update_connect_button(ui, &status.backend_state);

    let tailnet_name = status
        .current_tailnet
        .as_ref()
        .map(|tailnet| tailnet.name.as_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(&status.magic_dns_suffix);
    ui.title.set_subtitle(tailnet_name);

    match &status.this_node {
        Some(node) => {
            ui.self_row
                .set_title(&glib::markup_escape_text(&node.display_name()));
            let mut subtitle = node
                .primary_ip()
                .map(|ip| format!("{} · {ip}", os_label(&node.os)))
                .unwrap_or_else(|| os_label(&node.os));
            if let Some(owner) = status.owner_label(node) {
                subtitle.push_str(" · ");
                subtitle.push_str(&owner);
            }
            ui.self_row
                .set_subtitle(&glib::markup_escape_text(&subtitle));
        }
        None => {
            ui.self_row.set_title("This device");
            ui.self_row.set_subtitle("Not logged in");
        }
    }
    set_state_pill(ui, &status.backend_state, running);
    let peers = status.sorted_peers();
    apply_exit_node_status(ui, &peers);
    apply_device_list(ui, status, &peers);
    apply_local_capabilities(ui, status);

    if !status.health.is_empty() {
        show_hint(ui, &status.health.join("\n"));
    } else if !running {
        show_hint(
            ui,
            "Tailscale is disconnected. Click Connect to join your tailnet.",
        );
    }
}
