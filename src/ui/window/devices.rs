use std::rc::Rc;

use adw::prelude::*;

use crate::tailscale::{Node, Status};
use crate::ui::device_row;
use crate::util::{human_bytes, os_label};

use super::actions::toast;
use super::exit_nodes::set_exit_node;
use super::taildrop::pick_and_send;
use super::Ui;

pub(super) fn apply_device_list(ui: &Rc<Ui>, status: &Status, peers: &[Node]) {
    let shared_status = Rc::new(status.clone());
    ui.device_list.remove_all();
    for node in peers {
        ui.device_list
            .append(&build_device_row(ui, node, Rc::clone(&shared_status)));
    }
    let online = peers.iter().filter(|node| node.online).count();
    ui.devices_group
        .set_description(Some(&format!("{online} online · {} total", peers.len())));
}

fn build_device_row(ui: &Rc<Ui>, node: &Node, status: Rc<Status>) -> adw::ActionRow {
    let (row, send_button) = device_row::build_row(node, &status);

    {
        let ui = Rc::clone(ui);
        let node = node.clone();
        let status = Rc::clone(&status);
        let parent = row.clone();
        row.connect_activated(move |_| show_device_details(&ui, &parent, &node, &status));
    }

    if let (Some(button), Some(target)) = (send_button, node.cli_target()) {
        let ui = Rc::clone(ui);
        let target = target.to_string();
        let name = node.display_name();
        button.connect_clicked(move |_| pick_and_send(&ui, target.clone(), name.clone()));
    }
    row
}

fn show_device_details<P>(ui: &Rc<Ui>, parent: &P, node: &Node, status: &Status)
where
    P: IsA<gtk::Widget>,
{
    let mut rows = Vec::new();
    if let Some(owner) = status.owner_label(node) {
        push(&mut rows, "Owner", owner);
    }
    push(&mut rows, "OS", os_label(&node.os));
    push(
        &mut rows,
        "Status",
        format!(
            "{}{}",
            if node.online { "Online" } else { "Offline" },
            if node.active { " · active" } else { "" }
        ),
    );
    if !node.clean_dns_name().is_empty() {
        push(&mut rows, "DNS", node.clean_dns_name());
    }
    push(
        &mut rows,
        "Tailscale IPs",
        if node.tailscale_ips.is_empty() {
            "None".into()
        } else {
            node.tailscale_ips.join(", ")
        },
    );
    if !node.allowed_ips.is_empty() {
        push(&mut rows, "Allowed IPs", node.allowed_ips.join(", "));
    }
    for (key, value) in [
        ("Endpoint", &node.cur_addr),
        ("Relay", &node.relay),
        ("Last Seen", &node.last_seen),
        ("Last Handshake", &node.last_handshake),
        ("Key Expiry", &node.key_expiry),
    ] {
        if !value.is_empty() {
            push(&mut rows, key, value);
        }
    }
    if node.exit_node {
        push(&mut rows, "Exit Node", "Currently selected");
    } else if node.exit_node_option {
        push(&mut rows, "Exit Node", "Available");
    }
    if node.is_subnet_router() {
        push(&mut rows, "Subnet Router", "Yes");
    }

    let same_owner = status.has_same_owner(node);
    let taildrop = if status.can_send_taildrop_to(node) {
        "Available".into()
    } else if !same_owner {
        "Unavailable: different Tailscale user".into()
    } else if node.no_file_sharing_reason.is_empty() {
        "Unavailable".into()
    } else {
        node.no_file_sharing_reason.clone()
    };
    push(&mut rows, "Taildrop", taildrop);
    push(
        &mut rows,
        "Traffic",
        format!(
            "{} received / {} sent",
            human_bytes(node.rx_bytes),
            human_bytes(node.tx_bytes)
        ),
    );

    let popover = gtk::Popover::builder()
        .autohide(true)
        .has_arrow(true)
        .width_request(430)
        .build();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.add_css_class("detail-popover");

    let title = gtk::Label::new(Some(&node.display_name()));
    title.add_css_class("title-2");
    title.set_wrap(true);
    title.set_margin_top(16);
    title.set_margin_bottom(8);
    title.set_margin_start(16);
    title.set_margin_end(16);
    content.append(&title);

    let detail_list = gtk::Box::new(gtk::Orientation::Vertical, 0);
    for (key, value) in &rows {
        detail_list.append(&detail_row(key, value));
    }
    content.append(
        &gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .max_content_height(360)
            .child(&detail_list)
            .build(),
    );

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.set_margin_top(12);
    actions.set_margin_bottom(16);
    actions.set_margin_start(16);
    actions.set_margin_end(16);

    let copy_button = gtk::Button::with_label("Copy Details");
    copy_button.set_hexpand(true);
    actions.append(&copy_button);
    let send_button = status.can_send_taildrop_to(node).then(|| {
        let button = gtk::Button::with_label("Send File");
        button.add_css_class("suggested-action");
        button.set_hexpand(true);
        actions.append(&button);
        button
    });
    let exit_button = node.exit_node_option.then(|| {
        let button = gtk::Button::with_label("Use Exit Node");
        button.set_hexpand(true);
        actions.append(&button);
        button
    });
    content.append(&actions);
    popover.set_child(Some(&content));

    let target = node.cli_target().map(str::to_string);
    let name = node.display_name();
    let details = rows
        .iter()
        .map(|(key, value)| format!("{key}: {value}"))
        .collect::<Vec<_>>()
        .join("\n");
    let ui_for_copy = Rc::clone(ui);
    copy_button.connect_clicked(move |_| {
        ui_for_copy.window.clipboard().set_text(&details);
        toast(&ui_for_copy, "Device details copied");
    });

    if let Some(button) = send_button {
        let ui = Rc::clone(ui);
        let target = target.clone();
        let name = name.clone();
        button.connect_clicked(move |_| {
            if let Some(target) = &target {
                pick_and_send(&ui, target.clone(), name.clone());
            }
        });
    }
    if let Some(button) = exit_button {
        let ui = Rc::clone(ui);
        button.connect_clicked(move |_| {
            if let Some(target) = &target {
                set_exit_node(&ui, target.clone(), name.clone());
            }
        });
    }

    popover.set_parent(parent);
    popover.popup();
}

fn push(rows: &mut Vec<(&'static str, String)>, key: &'static str, value: impl Into<String>) {
    rows.push((key, value.into()));
}

fn detail_row(key: &str, value: &str) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.add_css_class("detail-row");
    row.set_margin_start(16);
    row.set_margin_end(16);
    row.set_margin_top(4);
    row.set_margin_bottom(4);

    let key_label = gtk::Label::new(Some(key));
    key_label.add_css_class("dim-label");
    key_label.add_css_class("detail-key");
    key_label.set_xalign(0.0);
    key_label.set_width_chars(14);
    key_label.set_valign(gtk::Align::Start);

    let value_label = gtk::Label::new(Some(value));
    value_label.set_xalign(0.0);
    value_label.set_wrap(true);
    value_label.set_selectable(true);
    value_label.set_hexpand(true);

    row.append(&key_label);
    row.append(&value_label);
    row
}
