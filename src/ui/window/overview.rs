use std::rc::Rc;

use crate::ui::dialogs;

use super::actions::show_error;
use super::Ui;

pub(super) fn show_overview(ui: &Rc<Ui>) {
    let Some(status) = ui.last_status.borrow().clone() else {
        show_error(ui, "Status has not loaded yet");
        return;
    };

    let mut rows = vec![("State", status.backend_state.label())];
    if let Some(node) = &status.this_node {
        push(&mut rows, "Device", node.display_name());
        if let Some(ip) = node.primary_ip() {
            push(&mut rows, "Tailscale IP", ip);
        }
        if !node.clean_dns_name().is_empty() {
            push(&mut rows, "DNS", node.clean_dns_name());
        }
        if let Some(owner) = status.owner_label(node) {
            push(&mut rows, "Signed in as", owner);
        }
    }

    if let Some(tailnet) = &status.current_tailnet {
        if !tailnet.name.is_empty() {
            push(&mut rows, "Tailnet", &tailnet.name);
        }
        if !tailnet.magic_dns_suffix.is_empty() {
            push(&mut rows, "MagicDNS", &tailnet.magic_dns_suffix);
        }
        push(
            &mut rows,
            "MagicDNS enabled",
            if tailnet.magic_dns_enabled {
                "yes"
            } else {
                "no"
            },
        );
    } else if !status.magic_dns_suffix.is_empty() {
        push(&mut rows, "MagicDNS", &status.magic_dns_suffix);
    }

    push(
        &mut rows,
        "Tailscale Version",
        if status.display_version().is_empty() {
            "unknown"
        } else {
            status.display_version()
        },
    );
    push(
        &mut rows,
        "Health",
        if status.health.is_empty() {
            "OK".into()
        } else {
            status.health.join("\n")
        },
    );

    let body = rows
        .into_iter()
        .map(|(key, value)| format!("{key}: {value}"))
        .collect::<Vec<_>>()
        .join("\n");
    dialogs::show_copyable(&ui.window, "TailScout Overview", &body);
}

fn push(rows: &mut Vec<(&'static str, String)>, key: &'static str, value: impl Into<String>) {
    rows.push((key, value.into()));
}
