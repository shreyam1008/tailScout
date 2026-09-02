//! Builds a single device row for the tailnet device list.

use adw::prelude::*;
use gtk::glib;

use crate::tailscale::{Node, Status};
use crate::util::os_label;

/// Build an `ActionRow` for a node. Returns the row plus an optional "send
/// files" button (present only for online peers) so the caller can wire it up.
pub fn build_row(node: &Node, status: &Status) -> (adw::ActionRow, Option<gtk::Button>) {
    let mut subtitle = match node.primary_ip() {
        Some(ip) => format!("{} · {}", os_label(&node.os), ip),
        None => os_label(&node.os),
    };
    if let Some(owner) = status.owner_label(node) {
        subtitle.push_str(" · ");
        subtitle.push_str(&owner);
    }
    if node.is_subnet_router() {
        subtitle.push_str(" · subnet router");
    }
    if node.exit_node_option {
        subtitle.push_str(" · exit node");
    }

    let row = adw::ActionRow::builder()
        .title(glib::markup_escape_text(&node.display_name()))
        .subtitle(glib::markup_escape_text(&subtitle))
        .activatable(true)
        .build();

    let icon_name = if node.online {
        "network-wireless-symbolic"
    } else {
        "network-offline-symbolic"
    };
    let icon = gtk::Image::from_icon_name(icon_name);
    icon.add_css_class(if node.online { "online" } else { "offline" });
    row.add_prefix(&icon);

    let status_label = gtk::Label::new(Some(if node.online { "Online" } else { "Offline" }));
    status_label.add_css_class("status-pill");
    status_label.add_css_class(if node.online { "online" } else { "offline" });
    status_label.set_valign(gtk::Align::Center);
    row.add_suffix(&status_label);

    let send_button = if status.can_send_taildrop_to(node) {
        let button = gtk::Button::from_icon_name("document-send-symbolic");
        button.add_css_class("flat");
        button.set_valign(gtk::Align::Center);
        button.set_tooltip_text(Some("Send File via Taildrop"));
        row.add_suffix(&button);
        Some(button)
    } else {
        if !status.has_same_owner(node) {
            row.set_subtitle(&glib::markup_escape_text(&format!(
                "{subtitle} · Taildrop unavailable: different Tailscale user"
            )));
        } else if !node.no_file_sharing_reason.is_empty() {
            row.set_subtitle(&glib::markup_escape_text(&format!(
                "{subtitle} · Taildrop unavailable: {}",
                node.no_file_sharing_reason
            )));
        }
        None
    };

    (row, send_button)
}
