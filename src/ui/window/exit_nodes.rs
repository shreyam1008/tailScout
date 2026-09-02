use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};

use crate::tailscale::{self, Node, Status};

use super::actions::{handle_backend_error, run_action, show_error};
use super::Ui;

pub(super) fn apply_local_capabilities(ui: &Rc<Ui>, status: &Status) {
    let advertising = status
        .this_node
        .as_ref()
        .is_some_and(|node| node.exit_node_option);
    ui.advertise_exit_row.set_subtitle(if advertising {
        "Currently advertising exit-node capability"
    } else {
        "Requires tailnet admin approval before other devices can use it"
    });
}

pub(super) fn apply_exit_node_status(ui: &Rc<Ui>, peers: &[Node]) {
    let options = peers.iter().filter(|node| node.exit_node_option).count();
    let selected = peers
        .iter()
        .find(|node| node.exit_node)
        .map(Node::display_name);

    match selected {
        Some(name) => {
            ui.exit_row.set_title(&format!("Using {name}"));
            ui.exit_row
                .set_subtitle("All regular internet traffic routes through this device");
        }
        None if options == 0 => {
            ui.exit_row.set_title("No exit node selected");
            ui.exit_row
                .set_subtitle("No peers are currently approved as exit nodes");
        }
        None => {
            ui.exit_row.set_title("No exit node selected");
            ui.exit_row
                .set_subtitle(&format!("{options} available · click to choose"));
        }
    }
}

pub(super) fn show_exit_node_dialog(ui: &Rc<Ui>) {
    let ui = Rc::clone(ui);
    glib::spawn_future_local(async move {
        match gio::spawn_blocking(tailscale::fetch_status).await {
            Ok(Ok(status)) => present_exit_nodes(&ui, status.exit_node_options()),
            Ok(Err(err)) => handle_backend_error(&ui, &err),
            Err(_) => show_error(&ui, "Could not load exit node information"),
        }
    });
}

fn present_exit_nodes(ui: &Rc<Ui>, options: Vec<Node>) {
    let dialog = adw::AlertDialog::new(
        Some("Exit Node"),
        Some("An exit node routes regular internet traffic through a Tailscale device."),
    );
    dialog.add_response("close", "Close");
    dialog.add_response("clear", "Clear Exit Node");
    for (index, node) in options.iter().enumerate() {
        dialog.add_response(
            &format!("use-{index}"),
            &format!("Use {}", node.display_name()),
        );
    }
    dialog.set_default_response(Some("close"));

    let ui_for_response = Rc::clone(ui);
    dialog.connect_response(None, move |_, response| {
        if response == "clear" {
            clear_exit_node(&ui_for_response);
            return;
        }
        let Some(node) = response
            .strip_prefix("use-")
            .and_then(|value| value.parse::<usize>().ok())
            .and_then(|index| options.get(index))
        else {
            return;
        };
        if let Some(target) = node.cli_target() {
            set_exit_node(&ui_for_response, target.to_string(), node.display_name());
        }
    });
    dialog.present(Some(&ui.window));
}

pub(super) fn set_exit_node(ui: &Rc<Ui>, target: String, name: String) {
    run_action(ui, format!("Using {name} as exit node"), move || {
        tailscale::set_exit_node(&target)
    });
}

fn clear_exit_node(ui: &Rc<Ui>) {
    run_action(ui, "Exit node cleared".into(), tailscale::clear_exit_node);
}

pub(super) fn show_advertise_exit_node_dialog(ui: &Rc<Ui>) {
    let dialog = adw::AlertDialog::new(
        Some("Advertise this device as an exit node?"),
        Some("This advertises that this computer can route internet traffic. Tailnet admin approval is still required."),
    );
    for (id, label) in [
        ("cancel", "Cancel"),
        ("disable", "Disable"),
        ("enable", "Enable"),
    ] {
        dialog.add_response(id, label);
    }
    dialog.set_response_appearance("enable", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("cancel"));

    let ui_for_response = Rc::clone(ui);
    dialog.connect_response(None, move |_, response| match response {
        "enable" => run_action(
            &ui_for_response,
            "This device is advertising exit-node capability".into(),
            || tailscale::advertise_exit_node(true),
        ),
        "disable" => run_action(
            &ui_for_response,
            "Exit-node advertising disabled".into(),
            || tailscale::advertise_exit_node(false),
        ),
        _ => {}
    });
    dialog.present(Some(&ui.window));
}
