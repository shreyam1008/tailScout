use std::env;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};

use crate::tailscale::{self, BackendState, TailscaleError};
use crate::ui::dialogs;
use crate::ui::help::{CLI_CHEATSHEET, GUIDE};

use super::accounts::login_account;
use super::taildrop::show_preferences;
use super::{apply_status, Ui, ADMIN_CONSOLE_URL};

#[derive(Clone, Copy)]
enum ConnectionAction {
    Login,
    Connect,
    Disconnect,
}

pub(super) fn refresh(ui: &Rc<Ui>) {
    if ui.busy.get() {
        return;
    }
    set_busy(ui, true);

    let ui = Rc::clone(ui);
    glib::spawn_future_local(async move {
        match gio::spawn_blocking(tailscale::fetch_status).await {
            Ok(Ok(status)) => apply_status(&ui, &status),
            Ok(Err(err)) => handle_backend_error(&ui, &err),
            Err(_) => show_error(&ui, "Failed to query Tailscale (background task error)"),
        }
        set_busy(&ui, false);
    });
}

pub(super) fn toggle_connection(ui: &Rc<Ui>) {
    if ui.busy.get() {
        return;
    }
    let action = match &*ui.backend_state.borrow() {
        BackendState::NeedsLogin => ConnectionAction::Login,
        BackendState::Running => ConnectionAction::Disconnect,
        _ => ConnectionAction::Connect,
    };
    set_busy(ui, true);

    let ui = Rc::clone(ui);
    glib::spawn_future_local(async move {
        let result = gio::spawn_blocking(move || match action {
            ConnectionAction::Login => tailscale::login().map(Some),
            ConnectionAction::Connect => tailscale::connect().map(|_| None),
            ConnectionAction::Disconnect => tailscale::disconnect().map(|_| None),
        })
        .await;

        match result {
            Ok(Ok(Some(output))) => show_login_output(&ui, &output),
            Ok(Ok(None)) => toast(
                &ui,
                match action {
                    ConnectionAction::Login => "Log in started",
                    ConnectionAction::Connect => "Connected",
                    ConnectionAction::Disconnect => "Disconnected",
                },
            ),
            Ok(Err(err)) => handle_action_error(&ui, &err),
            Err(_) => show_copyable_error(&ui, "Action failed", "Background task error"),
        }
        set_busy(&ui, false);
        refresh(&ui);
    });
}

pub(super) fn setup_help_menu(ui: &Rc<Ui>) {
    let menu = gio::Menu::new();
    for (label, action) in [
        ("TailScout Guide", "guide"),
        ("CLI Cheatsheet", "cheatsheet"),
        ("Network Check", "netcheck"),
        ("Exit Node List", "exit-node-list"),
        ("Open Admin Console", "admin-console"),
        ("Preferences", "preferences"),
        ("Tailscale Version", "version"),
        ("Bug Report", "bugreport"),
        ("About TailScout", "about"),
    ] {
        menu.append(Some(label), Some(&format!("win.{action}")));
    }
    ui.help_button.set_menu_model(Some(&menu));

    add_window_action(ui, "guide", |ui| {
        dialogs::show_copyable(&ui.window, "TailScout Guide", GUIDE)
    });
    add_window_action(ui, "cheatsheet", |ui| {
        dialogs::show_copyable(&ui.window, "TailScout CLI Cheatsheet", CLI_CHEATSHEET)
    });
    add_window_action(ui, "netcheck", |ui| {
        run_output_action(ui, "Network Check", tailscale::netcheck)
    });
    add_window_action(ui, "exit-node-list", |ui| {
        run_output_action(ui, "Exit Nodes", tailscale::exit_node_list)
    });
    add_window_action(ui, "admin-console", |ui| {
        dialogs::open_uri(&ui.window, ADMIN_CONSOLE_URL)
    });
    add_window_action(ui, "preferences", show_preferences);
    add_window_action(ui, "version", |ui| {
        run_output_action(ui, "Tailscale Version", tailscale::version)
    });
    add_window_action(ui, "bugreport", |ui| {
        run_output_action(ui, "Bug Report", tailscale::bugreport)
    });
    add_window_action(ui, "about", |ui| {
        let dialog = adw::AboutDialog::builder()
            .application_name("TailScout")
            .application_icon("dev.shre.TailScout")
            .developer_name("Shreyam Adhikari")
            .version(env!("CARGO_PKG_VERSION"))
            .website(env!("CARGO_PKG_HOMEPAGE"))
            .issue_url("https://github.com/shreyam1008/tailScout/issues")
            .copyright("© 2026 Shreyam Adhikari — MIT License")
            .comments("Native Rust + GTK4/libadwaita GUI for Tailscale on Linux.")
            .build();
        dialog.present(Some(&ui.window));
    });
}

fn add_window_action(ui: &Rc<Ui>, name: &str, callback: fn(&Rc<Ui>)) {
    let action = gio::SimpleAction::new(name, None);
    let ui_for_action = Rc::clone(ui);
    action.connect_activate(move |_, _| callback(&ui_for_action));
    ui.window.add_action(&action);
}

pub(super) fn setup_operator(ui: &Rc<Ui>) {
    let dialog = adw::AlertDialog::new(
        Some("Allow TailScout to control Tailscale?"),
        Some(&format!(
            "Tailscale actions need either root or operator permission. TailScout can open the system password prompt once and run:\n\npkexec {}\n\nAfter this, normal actions should work without starting TailScout with sudo.",
            operator_command()
        )),
    );
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("setup", "Open Password Prompt");
    dialog.set_response_appearance("setup", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("setup"));

    let ui_for_response = Rc::clone(ui);
    dialog.connect_response(None, move |_, response| {
        if response == "setup" {
            run_action(
                &ui_for_response,
                "Operator permission configured".into(),
                tailscale::set_operator_to_current_user,
            );
        }
    });
    dialog.present(Some(&ui.window));
}

pub(super) fn run_action<F>(ui: &Rc<Ui>, success_message: String, action: F)
where
    F: FnOnce() -> tailscale::Result<()> + Send + 'static,
{
    if ui.busy.get() {
        return;
    }
    set_busy(ui, true);

    let ui = Rc::clone(ui);
    glib::spawn_future_local(async move {
        match gio::spawn_blocking(action).await {
            Ok(Ok(())) => toast(&ui, &success_message),
            Ok(Err(err)) => handle_action_error(&ui, &err),
            Err(_) => show_copyable_error(&ui, "Action failed", "Background task error"),
        }
        set_busy(&ui, false);
        refresh(&ui);
    });
}

pub(super) fn run_output_action<F>(ui: &Rc<Ui>, title: &str, action: F)
where
    F: FnOnce() -> tailscale::Result<String> + Send + 'static,
{
    if ui.busy.get() {
        return;
    }
    set_busy(ui, true);

    let title = title.to_string();
    let ui = Rc::clone(ui);
    glib::spawn_future_local(async move {
        match gio::spawn_blocking(action).await {
            Ok(Ok(output)) => dialogs::show_copyable(
                &ui.window,
                &title,
                if output.trim().is_empty() {
                    "Command completed successfully."
                } else {
                    &output
                },
            ),
            Ok(Err(err)) => handle_action_error(&ui, &err),
            Err(_) => show_copyable_error(&ui, "Command failed", "Background task error"),
        }
        set_busy(&ui, false);
    });
}

pub(super) fn set_busy(ui: &Rc<Ui>, busy: bool) {
    ui.busy.set(busy);
    ui.spinner.set_visible(busy);
    if busy {
        ui.spinner.start();
    } else {
        ui.spinner.stop();
    }
    for widget in [
        ui.refresh_button.upcast_ref::<gtk::Widget>(),
        ui.connect_button.upcast_ref(),
        ui.setup_button.upcast_ref(),
        ui.receive_button.upcast_ref(),
        ui.admin_button.upcast_ref(),
        ui.profiles_button.upcast_ref(),
        ui.help_button.upcast_ref(),
        ui.admin_row.upcast_ref(),
        ui.taildrop_row.upcast_ref(),
        ui.taildrop_folder_row.upcast_ref(),
        ui.exit_row.upcast_ref(),
        ui.advertise_exit_row.upcast_ref(),
        ui.device_list.upcast_ref(),
    ] {
        widget.set_sensitive(!busy);
    }
}

pub(super) fn update_connect_button(ui: &Rc<Ui>, state: &BackendState) {
    let (label, add, remove) = match state {
        BackendState::NeedsLogin => ("Log In", "suggested-action", "destructive-action"),
        BackendState::Running => ("Disconnect", "destructive-action", "suggested-action"),
        _ => ("Connect", "suggested-action", "destructive-action"),
    };
    ui.connect_button.set_label(label);
    ui.connect_button.remove_css_class(remove);
    ui.connect_button.add_css_class(add);
}

pub(super) fn set_state_pill(ui: &Rc<Ui>, state: &BackendState, running: bool) {
    ui.state_label.set_text(&state.label());
    ui.state_label.remove_css_class("online");
    ui.state_label.remove_css_class("offline");
    ui.state_label
        .add_css_class(if running { "online" } else { "offline" });
}

pub(super) fn handle_backend_error(ui: &Rc<Ui>, err: &TailscaleError) {
    if err.is_permission_problem() {
        show_permission_dialog(ui, &err.to_string());
    } else {
        show_error(ui, &err.to_string());
    }
}

pub(super) fn handle_action_error(ui: &Rc<Ui>, err: &TailscaleError) {
    if err.is_permission_problem() {
        show_permission_dialog(ui, &err.to_string());
    } else if err.is_taildrop_different_user_problem() {
        show_copyable_error(
            ui,
            "Taildrop cannot send to this device",
            &format!(
                "The peer belongs to a different Tailscale user, so Taildrop refused the transfer. Use another transfer method or a device owned by the current user.\n\nOriginal error:\n{err}"
            ),
        );
    } else {
        show_copyable_error(ui, "Tailscale command failed", &err.to_string());
    }
}

fn show_permission_dialog(ui: &Rc<Ui>, detail: &str) {
    show_error(
        ui,
        "Permission needed. TailScout can open the system password prompt to fix operator access.",
    );
    let dialog = adw::AlertDialog::new(
        Some("Permission needed"),
        Some(&format!(
            "This Linux user cannot control tailscaled directly.\n\nFix Permission runs:\n\npkexec {}\n\nOr run this manually:\n\nsudo {}\n\nOriginal error:\n{detail}",
            operator_command(),
            operator_command()
        )),
    );
    for (id, label) in [
        ("cancel", "Cancel"),
        ("details", "Details"),
        ("copy", "Copy Command"),
        ("login", "Try Log In"),
        ("admin", "Admin Console"),
        ("setup", "Fix Permission"),
    ] {
        dialog.add_response(id, label);
    }
    dialog.set_response_appearance("setup", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("setup"));

    let ui_for_response = Rc::clone(ui);
    let detail = detail.to_string();
    dialog.connect_response(None, move |_, response| match response {
        "setup" => setup_operator(&ui_for_response),
        "copy" => {
            ui_for_response
                .window
                .clipboard()
                .set_text(&format!("sudo {}", operator_command()));
            toast(&ui_for_response, "Operator command copied");
        }
        "login" => login_account(&ui_for_response),
        "admin" => dialogs::open_uri(&ui_for_response.window, ADMIN_CONSOLE_URL),
        "details" => show_copyable_error(&ui_for_response, "Permission details", &detail),
        _ => {}
    });
    dialog.present(Some(&ui.window));
}

fn operator_command() -> String {
    let user = env::var("USER").unwrap_or_else(|_| "$USER".into());
    format!("tailscale set --operator={user}")
}

pub(super) fn show_copyable_error(ui: &Rc<Ui>, title: &str, body: &str) {
    show_error(ui, title);
    dialogs::show_copyable(&ui.window, title, body);
}

fn show_login_output(ui: &Rc<Ui>, output: &str) {
    if let Some(url) = dialogs::first_url(output) {
        dialogs::open_uri(&ui.window, &url);
        toast(ui, "Log in link opened in browser");
    }
    dialogs::show_copyable(&ui.window, "Tailscale Log In", output);
}

pub(super) fn show_error(ui: &Rc<Ui>, message: &str) {
    ui.banner.set_title(message);
    ui.banner.add_css_class("error");
    ui.banner.set_revealed(true);
}

pub(super) fn show_hint(ui: &Rc<Ui>, message: &str) {
    ui.banner.set_title(message);
    ui.banner.remove_css_class("error");
    ui.banner.set_revealed(true);
}

pub(super) fn toast(ui: &Rc<Ui>, message: &str) {
    ui.toasts.add_toast(adw::Toast::new(message));
}
