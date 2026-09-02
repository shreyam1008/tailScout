use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};

use crate::tailscale::{self, Profile};

use super::actions::{run_action, run_output_action, setup_operator, show_error};
use super::Ui;

pub(super) fn login_account(ui: &Rc<Ui>) {
    run_output_action(ui, "Tailscale Log In", tailscale::login);
}

fn confirm_logout(ui: &Rc<Ui>) {
    let dialog = adw::AlertDialog::new(
        Some("Log out from this Tailscale account?"),
        Some("This disconnects Tailscale and expires this machine's node key. You will need to log in again later."),
    );
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("logout", "Log Out");
    dialog.set_response_appearance("logout", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));

    let ui_for_response = Rc::clone(ui);
    dialog.connect_response(None, move |_, response| {
        if response == "logout" {
            run_action(
                &ui_for_response,
                "Logged out of Tailscale".into(),
                tailscale::logout,
            );
        }
    });
    dialog.present(Some(&ui.window));
}

pub(super) fn show_profiles(ui: &Rc<Ui>) {
    let ui = Rc::clone(ui);
    glib::spawn_future_local(async move {
        match gio::spawn_blocking(tailscale::profiles).await {
            Ok(Ok(profiles)) => show_profiles_dialog(&ui, profiles),
            Ok(Err(err)) => show_accounts_error_dialog(&ui, &err.to_string()),
            Err(_) => show_error(&ui, "Could not load accounts"),
        }
    });
}

fn show_accounts_error_dialog(ui: &Rc<Ui>, detail: &str) {
    let dialog = adw::AlertDialog::new(
        Some("Accounts unavailable"),
        Some(&format!(
            "This Linux user cannot read saved Tailscale profiles yet. Fix Permission opens the system password prompt once. You can still log in or out.\n\n{detail}"
        )),
    );
    for (id, label) in [
        ("close", "Close"),
        ("login", "Log In"),
        ("logout", "Log Out"),
        ("fix", "Fix Permission"),
    ] {
        dialog.add_response(id, label);
    }
    dialog.set_default_response(Some("close"));

    let ui_for_response = Rc::clone(ui);
    dialog.connect_response(None, move |_, response| match response {
        "login" => login_account(&ui_for_response),
        "logout" => confirm_logout(&ui_for_response),
        "fix" => setup_operator(&ui_for_response),
        _ => {}
    });
    dialog.present(Some(&ui.window));
}

fn show_profiles_dialog(ui: &Rc<Ui>, profiles: Vec<Profile>) {
    let body = if profiles.is_empty() {
        "No saved Tailscale accounts found.".into()
    } else {
        profiles
            .iter()
            .map(|profile| {
                format!(
                    "{}{}",
                    if profile.selected { "✓ " } else { "" },
                    profile.display_name()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let dialog = adw::AlertDialog::new(Some("Accounts and Tailnets"), Some(&body));
    dialog.add_response("close", "Close");
    dialog.add_response("login", "Add / Log In Account");
    dialog.add_response("logout", "Log Out Current");
    for (index, profile) in profiles
        .iter()
        .enumerate()
        .filter(|(_, item)| !item.selected)
    {
        dialog.add_response(
            &format!("switch-{index}"),
            &format!("Switch to {}", profile.display_name()),
        );
    }
    dialog.set_default_response(Some("close"));

    let ui_for_response = Rc::clone(ui);
    dialog.connect_response(None, move |_, response| match response {
        "login" => login_account(&ui_for_response),
        "logout" => confirm_logout(&ui_for_response),
        _ => {
            let Some(profile) = response
                .strip_prefix("switch-")
                .and_then(|value| value.parse::<usize>().ok())
                .and_then(|index| profiles.get(index))
            else {
                return;
            };
            let id = profile.switch_key();
            let name = profile.display_name();
            run_action(&ui_for_response, format!("Switched to {name}"), move || {
                tailscale::switch_profile(&id)
            });
        }
    });
    dialog.present(Some(&ui.window));
}
