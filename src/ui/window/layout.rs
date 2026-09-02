use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;

use crate::settings;
use crate::tailscale::BackendState;

use super::{Ui, WINDOW_HEIGHT, WINDOW_WIDTH};

pub(super) fn build(app: &adw::Application) -> Rc<Ui> {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(WINDOW_WIDTH)
        .default_height(WINDOW_HEIGHT)
        .width_request(380)
        .height_request(500)
        .build();
    let title = adw::WindowTitle::new("TailScout", "Small native Tailscale control");

    let connect_button = gtk::Button::with_label("Connect");
    connect_button.add_css_class("suggested-action");
    let setup_button = icon_button(
        "system-lock-screen-symbolic",
        "Set current user as Tailscale operator",
    );
    let admin_button = gtk::Button::with_label("Open");
    admin_button.set_tooltip_text(Some("Open Tailscale admin console"));
    let receive_button = icon_button("folder-download-symbolic", "Receive Taildrop files");
    let profiles_button = icon_button("avatar-default-symbolic", "Accounts and Tailnets");
    let help_button = gtk::MenuButton::new();
    help_button.set_icon_name("open-menu-symbolic");
    help_button.set_tooltip_text(Some("Help, diagnostics, and about"));
    let refresh_button = icon_button("view-refresh-symbolic", "Refresh");
    let spinner = gtk::Spinner::new();
    spinner.set_visible(false);

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&title));
    header.pack_start(&connect_button);
    for widget in [
        refresh_button.upcast_ref::<gtk::Widget>(),
        profiles_button.upcast_ref(),
        receive_button.upcast_ref(),
        setup_button.upcast_ref(),
        help_button.upcast_ref(),
        spinner.upcast_ref(),
    ] {
        header.pack_end(widget);
    }

    let banner = adw::Banner::new("");
    banner.set_revealed(false);

    let self_row = action_row("Overview", "Connection, identity, and daemon state", true);
    let state_label = gtk::Label::new(Some("…"));
    state_label.add_css_class("status-pill");
    state_label.set_valign(gtk::Align::Center);
    self_row.add_suffix(&state_label);

    let overview_group = group("Overview", None);
    overview_group.add(&self_row);
    let admin_row = action_row(
        "Admin Console",
        "Open Tailscale web settings, users, ACLs, and file-sharing policy",
        true,
    );
    admin_row.add_prefix(&gtk::Image::from_icon_name("web-browser-symbolic"));
    admin_row.add_suffix(&admin_button);
    overview_group.add(&admin_row);

    let taildrop_group = group(
        "Taildrop",
        Some("Receive files locally. Sending depends on Tailscale policy for the target."),
    );
    let taildrop_row = action_row(
        "Receive Files",
        "Move files from the Tailscale inbox into your chosen folder",
        true,
    );
    taildrop_row.add_prefix(&gtk::Image::from_icon_name("folder-download-symbolic"));
    taildrop_group.add(&taildrop_row);
    let taildrop_folder_row = action_row("Default receive folder", "", true);
    taildrop_folder_row.add_prefix(&gtk::Image::from_icon_name("folder-symbolic"));
    taildrop_group.add(&taildrop_folder_row);

    let exit_group = group(
        "Exit Node",
        Some("Route normal internet traffic through another Tailscale device."),
    );
    let exit_row = action_row(
        "No exit node selected",
        "No available exit-node peers yet",
        true,
    );
    exit_row.add_prefix(&gtk::Image::from_icon_name("network-vpn-symbolic"));
    exit_group.add(&exit_row);
    let advertise_exit_row = action_row(
        "Advertise This Device",
        "Requires tailnet admin approval before other devices can use it",
        true,
    );
    advertise_exit_row.add_prefix(&gtk::Image::from_icon_name("emblem-shared-symbolic"));
    exit_group.add(&advertise_exit_row);

    let device_list = gtk::ListBox::new();
    device_list.set_selection_mode(gtk::SelectionMode::None);
    device_list.add_css_class("boxed-list");
    let placeholder = gtk::Label::new(Some("No other devices"));
    placeholder.add_css_class("dim-label");
    placeholder.add_css_class("device-empty");
    device_list.set_placeholder(Some(&placeholder));
    let devices_group = group("Devices", None);
    devices_group.add(&device_list);

    let left = column("main-column", [&overview_group, &devices_group]);
    let right = column("side-column", [&taildrop_group, &exit_group]);
    let grid = gtk::Box::new(gtk::Orientation::Horizontal, 18);
    grid.add_css_class("main-grid");
    grid.append(&left);
    grid.append(&right);
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&grid)
        .vexpand(true)
        .build();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&banner);
    content.append(&scrolled);
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&content));
    let toasts = adw::ToastOverlay::new();
    toasts.set_child(Some(&toolbar));
    window.set_content(Some(&toasts));

    Rc::new(Ui {
        window,
        toasts,
        banner,
        title,
        connect_button,
        refresh_button,
        spinner,
        setup_button,
        admin_button,
        receive_button,
        profiles_button,
        help_button,
        self_row,
        state_label,
        taildrop_folder_row,
        exit_row,
        advertise_exit_row,
        devices_group,
        device_list,
        busy: Cell::new(false),
        file_dialog_open: Cell::new(false),
        backend_state: RefCell::new(BackendState::NeedsLogin),
        last_status: RefCell::new(None),
        settings: RefCell::new(settings::load()),
        admin_row,
        taildrop_row,
    })
}

fn icon_button(icon: &str, tooltip: &str) -> gtk::Button {
    let button = gtk::Button::from_icon_name(icon);
    button.set_tooltip_text(Some(tooltip));
    button
}

fn action_row(title: &str, subtitle: &str, activatable: bool) -> adw::ActionRow {
    adw::ActionRow::builder()
        .title(title)
        .subtitle(subtitle)
        .activatable(activatable)
        .build()
}

fn group(title: &str, description: Option<&str>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::new();
    group.set_title(title);
    group.set_description(description);
    group
}

fn column<'a>(
    class: &str,
    groups: impl IntoIterator<Item = &'a adw::PreferencesGroup>,
) -> gtk::Box {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 18);
    column.add_css_class(class);
    column.set_hexpand(true);
    for group in groups {
        column.append(group);
    }
    column
}
