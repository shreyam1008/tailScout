//! The main TailScout window: status, connection controls, profiles, Taildrop,
//! exit-node basics, and the device list.
//!
//! All blocking backend calls run on a worker thread via `gio::spawn_blocking`
//! and update the UI back on the GTK main loop, so the window never freezes.

use std::cell::{Cell, RefCell};
use std::env;
use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};

use crate::settings::{self, Settings};
use crate::tailscale::{self, BackendState, Node, Profile, Status, TailscaleError};
use crate::ui::help::{CLI_CHEATSHEET, GUIDE};
use crate::ui::{device_row, dialogs};
use crate::util::{human_bytes, os_label};

const WINDOW_WIDTH: i32 = 920;
const WINDOW_HEIGHT: i32 = 700;
const ADMIN_CONSOLE_URL: &str = "https://login.tailscale.com/admin/machines";

#[derive(Clone, Copy)]
enum ConnectionAction {
    Login,
    Connect,
    Disconnect,
}

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
    version_row: adw::ActionRow,
    tailnet_row: adw::ActionRow,
    user_row: adw::ActionRow,
    health_row: adw::ActionRow,
    taildrop_folder_row: adw::ActionRow,
    exit_row: adw::ActionRow,
    advertise_exit_row: adw::ActionRow,
    devices_group: adw::PreferencesGroup,
    device_list: gtk::ListBox,
    running: Cell<bool>,
    busy: Cell<bool>,
    backend_state: RefCell<BackendState>,
    last_status: RefCell<Option<Status>>,
    settings: RefCell<Settings>,
}

pub fn build_window(app: &adw::Application) -> adw::ApplicationWindow {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(WINDOW_WIDTH)
        .default_height(WINDOW_HEIGHT)
        .width_request(380)
        .height_request(500)
        .build();

    let title = adw::WindowTitle::new("TailScout", "Small native Tailscale control");
    let current_settings = settings::load();

    let connect_button = gtk::Button::with_label("Connect");
    connect_button.add_css_class("suggested-action");

    let setup_button = gtk::Button::from_icon_name("system-lock-screen-symbolic");
    setup_button.set_tooltip_text(Some("Set current user as Tailscale operator"));

    let admin_button = gtk::Button::from_icon_name("web-browser-symbolic");
    admin_button.set_tooltip_text(Some("Open Tailscale admin console"));

    let receive_button = gtk::Button::from_icon_name("folder-download-symbolic");
    receive_button.set_tooltip_text(Some("Receive Taildrop files"));

    let profiles_button = gtk::Button::from_icon_name("avatar-default-symbolic");
    profiles_button.set_tooltip_text(Some("Accounts and tailnets"));

    let help_button = gtk::MenuButton::new();
    help_button.set_icon_name("open-menu-symbolic");
    help_button.set_tooltip_text(Some("Help, diagnostics, and about"));

    let refresh_button = gtk::Button::from_icon_name("view-refresh-symbolic");
    refresh_button.set_tooltip_text(Some("Refresh"));

    let spinner = gtk::Spinner::new();
    spinner.set_visible(false);

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&title));
    header.pack_start(&connect_button);
    header.pack_end(&refresh_button);
    header.pack_end(&profiles_button);
    header.pack_end(&receive_button);
    header.pack_end(&admin_button);
    header.pack_end(&setup_button);
    header.pack_end(&help_button);
    header.pack_end(&spinner);

    let banner = adw::Banner::new("");
    banner.set_revealed(false);

    let self_row = adw::ActionRow::builder()
        .title("Overview")
        .subtitle("Connection, identity, and daemon state")
        .activatable(true)
        .build();
    let state_label = gtk::Label::new(Some("…"));
    state_label.add_css_class("status-pill");
    state_label.set_valign(gtk::Align::Center);
    self_row.add_suffix(&state_label);

    let version_row = adw::ActionRow::builder().title("Tailscale version").build();
    let tailnet_row = adw::ActionRow::builder().title("Tailnet").build();
    let user_row = adw::ActionRow::builder().title("Signed in as").build();
    let health_row = adw::ActionRow::builder().title("Health").build();

    let overview_group = adw::PreferencesGroup::new();
    overview_group.set_title("Overview");
    overview_group.add(&self_row);

    let taildrop_group = adw::PreferencesGroup::new();
    taildrop_group.set_title("Taildrop");
    taildrop_group.set_description(Some(
        "Transfer files between your devices without cloud storage.",
    ));

    let taildrop_row = adw::ActionRow::builder()
        .title("Receive files")
        .subtitle("Move files from the Tailscale inbox into your chosen folder")
        .activatable(true)
        .build();
    let taildrop_icon = gtk::Image::from_icon_name("folder-download-symbolic");
    taildrop_row.add_prefix(&taildrop_icon);
    taildrop_group.add(&taildrop_row);

    let taildrop_folder_row = adw::ActionRow::builder()
        .title("Default receive folder")
        .activatable(true)
        .build();
    let folder_icon = gtk::Image::from_icon_name("folder-symbolic");
    taildrop_folder_row.add_prefix(&folder_icon);
    taildrop_group.add(&taildrop_folder_row);

    let exit_group = adw::PreferencesGroup::new();
    exit_group.set_title("Exit node");
    exit_group.set_description(Some(
        "Route normal internet traffic through another Tailscale device.",
    ));

    let exit_row = adw::ActionRow::builder()
        .title("No exit node selected")
        .subtitle("No available exit-node peers yet")
        .activatable(true)
        .build();
    let exit_icon = gtk::Image::from_icon_name("network-vpn-symbolic");
    exit_row.add_prefix(&exit_icon);
    exit_group.add(&exit_row);

    let advertise_exit_row = adw::ActionRow::builder()
        .title("Offer this device as an exit node")
        .subtitle("Requires tailnet admin approval before other devices can use it")
        .activatable(true)
        .build();
    let advertise_icon = gtk::Image::from_icon_name("emblem-shared-symbolic");
    advertise_exit_row.add_prefix(&advertise_icon);
    exit_group.add(&advertise_exit_row);

    let device_list = gtk::ListBox::new();
    device_list.set_selection_mode(gtk::SelectionMode::None);
    device_list.add_css_class("boxed-list");

    let placeholder = gtk::Label::new(Some("No other devices"));
    placeholder.add_css_class("dim-label");
    placeholder.add_css_class("device-empty");
    device_list.set_placeholder(Some(&placeholder));

    let devices_group = adw::PreferencesGroup::new();
    devices_group.set_title("Devices");
    devices_group.add(&device_list);

    let left_column = gtk::Box::new(gtk::Orientation::Vertical, 18);
    left_column.add_css_class("main-column");
    left_column.set_hexpand(true);
    left_column.append(&overview_group);
    left_column.append(&taildrop_group);

    let right_column = gtk::Box::new(gtk::Orientation::Vertical, 18);
    right_column.add_css_class("side-column");
    right_column.set_hexpand(true);
    right_column.append(&devices_group);
    right_column.append(&exit_group);

    let main_grid = gtk::Box::new(gtk::Orientation::Horizontal, 18);
    main_grid.add_css_class("main-grid");
    main_grid.append(&left_column);
    main_grid.append(&right_column);

    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&main_grid)
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

    let ui = Rc::new(Ui {
        window: window.clone(),
        toasts,
        banner,
        title,
        connect_button: connect_button.clone(),
        refresh_button: refresh_button.clone(),
        spinner,
        setup_button: setup_button.clone(),
        admin_button: admin_button.clone(),
        receive_button: receive_button.clone(),
        profiles_button: profiles_button.clone(),
        help_button: help_button.clone(),
        self_row,
        state_label,
        version_row,
        tailnet_row,
        user_row,
        health_row,
        taildrop_folder_row,
        exit_row: exit_row.clone(),
        advertise_exit_row: advertise_exit_row.clone(),
        devices_group,
        device_list,
        running: Cell::new(false),
        busy: Cell::new(false),
        backend_state: RefCell::new(BackendState::NeedsLogin),
        last_status: RefCell::new(None),
        settings: RefCell::new(current_settings),
    });
    update_taildrop_folder_row(&ui);

    {
        let ui = ui.clone();
        refresh_button.connect_clicked(move |_| refresh(&ui));
    }
    {
        let ui = ui.clone();
        connect_button.connect_clicked(move |_| toggle_connection(&ui));
    }
    {
        let ui = ui.clone();
        setup_button.connect_clicked(move |_| setup_operator(&ui));
    }
    {
        let ui = ui.clone();
        admin_button.connect_clicked(move |_| dialogs::open_uri(&ui.window, ADMIN_CONSOLE_URL));
    }
    {
        let ui = ui.clone();
        let row = ui.self_row.clone();
        row.connect_activated(move |_| show_overview(&ui));
    }
    {
        let ui = ui.clone();
        receive_button.connect_clicked(move |_| receive_into_default_or_pick(&ui));
    }
    {
        let ui = ui.clone();
        taildrop_row.connect_activated(move |_| receive_into_default_or_pick(&ui));
    }
    {
        let ui = ui.clone();
        let row = ui.taildrop_folder_row.clone();
        row.connect_activated(move |_| show_preferences(&ui));
    }
    {
        let ui = ui.clone();
        profiles_button.connect_clicked(move |_| show_profiles(&ui));
    }
    {
        let ui = ui.clone();
        exit_row.connect_activated(move |_| show_exit_node_dialog(&ui));
    }
    {
        let ui = ui.clone();
        advertise_exit_row.connect_activated(move |_| show_advertise_exit_node_dialog(&ui));
    }
    setup_help_menu(&ui);

    refresh(&ui);
    window
}

fn refresh(ui: &Rc<Ui>) {
    if ui.busy.get() {
        return;
    }
    set_busy(ui, true);

    let ui = ui.clone();
    glib::spawn_future_local(async move {
        let result = gio::spawn_blocking(tailscale::fetch_status).await;
        set_busy(&ui, false);
        match result {
            Ok(Ok(status)) => apply_status(&ui, &status),
            Ok(Err(err)) => handle_backend_error(&ui, &err),
            Err(_) => show_error(&ui, "Failed to query Tailscale (background task error)"),
        }
    });
}

fn setup_help_menu(ui: &Rc<Ui>) {
    let menu = gio::Menu::new();
    menu.append(Some("TailScout Guide"), Some("win.guide"));
    menu.append(Some("CLI Cheatsheet"), Some("win.cheatsheet"));
    menu.append(Some("Run Netcheck"), Some("win.netcheck"));
    menu.append(Some("Exit Node CLI List"), Some("win.exit-node-list"));
    menu.append(Some("Open Admin Console"), Some("win.admin-console"));
    menu.append(Some("Preferences"), Some("win.preferences"));
    menu.append(Some("Tailscale Version"), Some("win.version"));
    menu.append(Some("Bug Report ID"), Some("win.bugreport"));
    menu.append(Some("About TailScout"), Some("win.about"));
    ui.help_button.set_menu_model(Some(&menu));

    add_window_action(ui, "guide", show_guide);
    add_window_action(ui, "cheatsheet", show_cli_cheatsheet);
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
    add_window_action(ui, "about", show_about);
}

fn add_window_action(ui: &Rc<Ui>, name: &str, callback: fn(&Rc<Ui>)) {
    let action = gio::SimpleAction::new(name, None);
    let ui_for_action = ui.clone();
    action.connect_activate(move |_, _| callback(&ui_for_action));
    ui.window.add_action(&action);
}

fn toggle_connection(ui: &Rc<Ui>) {
    if ui.busy.get() {
        return;
    }
    let action = match &*ui.backend_state.borrow() {
        BackendState::NeedsLogin => ConnectionAction::Login,
        BackendState::Running => ConnectionAction::Disconnect,
        _ => ConnectionAction::Connect,
    };
    set_busy(ui, true);

    let ui = ui.clone();
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
                    ConnectionAction::Login => "Login started",
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

fn apply_status(ui: &Rc<Ui>, status: &Status) {
    ui.banner.set_revealed(false);
    ui.last_status.replace(Some(status.clone()));

    let running = status.backend_state.is_running();
    ui.running.set(running);
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
                .map(|ip| format!("{} · {}", os_label(&node.os), ip))
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

    let version = if status.version.is_empty() {
        status.client_version.as_str()
    } else {
        status.version.as_str()
    };
    ui.version_row.set_subtitle(if version.is_empty() {
        "unknown"
    } else {
        version
    });

    if let Some(tailnet) = &status.current_tailnet {
        ui.tailnet_row.set_subtitle(&format!(
            "{} · MagicDNS {}",
            tailnet.magic_dns_suffix,
            if tailnet.magic_dns_enabled {
                "on"
            } else {
                "off"
            }
        ));
    } else {
        ui.tailnet_row.set_subtitle(&status.magic_dns_suffix);
    }

    let signed_in_as = status
        .this_node
        .as_ref()
        .and_then(|node| status.owner_label(node))
        .unwrap_or_else(|| "Unknown".to_string());
    ui.user_row.set_subtitle(&signed_in_as);

    if status.health.is_empty() {
        ui.health_row.set_subtitle("OK");
    } else {
        ui.health_row.set_subtitle(&status.health.join(" · "));
    }

    apply_exit_node_status(ui, status);
    apply_device_list(ui, status);
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

fn apply_local_capabilities(ui: &Rc<Ui>, status: &Status) {
    let is_exit = status
        .this_node
        .as_ref()
        .is_some_and(|node| node.exit_node_option);
    ui.advertise_exit_row.set_subtitle(if is_exit {
        "Currently advertising exit-node capability"
    } else {
        "Requires tailnet admin approval before other devices can use it"
    });
}

fn apply_exit_node_status(ui: &Rc<Ui>, status: &Status) {
    let options = status.exit_node_options();
    let selected = status
        .peers
        .iter()
        .find(|node| node.exit_node)
        .map(Node::display_name);

    if let Some(name) = selected {
        ui.exit_row.set_title(&format!("Using {name}"));
        ui.exit_row
            .set_subtitle("All regular internet traffic routes through this device");
    } else if options.is_empty() {
        ui.exit_row.set_title("No exit node selected");
        ui.exit_row
            .set_subtitle("No peers are currently approved as exit nodes");
    } else {
        ui.exit_row.set_title("No exit node selected");
        ui.exit_row
            .set_subtitle(&format!("{} available · click to choose", options.len()));
    }
}

fn apply_device_list(ui: &Rc<Ui>, status: &Status) {
    let peers = status.sorted_peers();
    ui.device_list.remove_all();
    for node in &peers {
        let row = build_device_row(ui, node, status);
        ui.device_list.append(&row);
    }
    let count = peers.len();
    let online = peers.iter().filter(|node| node.online).count();
    ui.devices_group
        .set_description(Some(&format!("{online} online · {count} total")));
}

fn build_device_row(ui: &Rc<Ui>, node: &Node, status: &Status) -> adw::ActionRow {
    let (row, send_button) = device_row::build_row(node, status);

    {
        let ui = ui.clone();
        let node = node.clone();
        let status = status.clone();
        let row_for_details = row.clone();
        row.connect_activated(move |_| show_device_details(&ui, &row_for_details, &node, &status));
    }

    if let (Some(button), Some(ip)) = (send_button, node.primary_ip()) {
        let ui = ui.clone();
        let target = ip.to_string();
        let name = node.display_name();
        button.connect_clicked(move |_| pick_and_send(&ui, target.clone(), name.clone()));
    }

    row
}

fn show_device_details<P>(ui: &Rc<Ui>, parent: &P, node: &Node, status: &Status)
where
    P: IsA<gtk::Widget>,
{
    let mut body = String::new();
    let mut rows: Vec<(String, String)> = Vec::new();
    if let Some(owner) = status.owner_label(node) {
        push_detail(&mut body, &mut rows, "Owner", owner);
    }
    push_detail(&mut body, &mut rows, "OS", os_label(&node.os));
    push_detail(
        &mut body,
        &mut rows,
        "Status",
        format!(
            "{}{}",
            if node.online { "Online" } else { "Offline" },
            if node.active { " · active" } else { "" }
        ),
    );
    if !node.clean_dns_name().is_empty() {
        push_detail(&mut body, &mut rows, "DNS", node.clean_dns_name());
    }
    if node.tailscale_ips.is_empty() {
        push_detail(&mut body, &mut rows, "Tailscale IPs", "none");
    } else {
        push_detail(
            &mut body,
            &mut rows,
            "Tailscale IPs",
            node.tailscale_ips.join(", "),
        );
    }
    if !node.allowed_ips.is_empty() {
        push_detail(
            &mut body,
            &mut rows,
            "Allowed IPs",
            node.allowed_ips.join(", "),
        );
    }
    if !node.cur_addr.is_empty() {
        push_detail(&mut body, &mut rows, "Endpoint", node.cur_addr.clone());
    }
    if !node.relay.is_empty() {
        push_detail(&mut body, &mut rows, "Relay", node.relay.clone());
    }
    if !node.last_seen.is_empty() {
        push_detail(&mut body, &mut rows, "Last seen", node.last_seen.clone());
    }
    if !node.last_handshake.is_empty() {
        push_detail(
            &mut body,
            &mut rows,
            "Last handshake",
            node.last_handshake.clone(),
        );
    }
    if !node.key_expiry.is_empty() {
        push_detail(&mut body, &mut rows, "Key expiry", node.key_expiry.clone());
    }
    if node.exit_node {
        push_detail(&mut body, &mut rows, "Exit node", "currently selected");
    } else if node.exit_node_option {
        push_detail(&mut body, &mut rows, "Exit node", "available");
    }
    if node.is_subnet_router() {
        push_detail(&mut body, &mut rows, "Subnet router", "yes");
    }
    let same_owner = status
        .this_node
        .as_ref()
        .map(|this_node| {
            this_node.user_id == 0 || node.user_id == 0 || this_node.user_id == node.user_id
        })
        .unwrap_or(true);
    let taildrop = if node.can_receive_taildrop() && same_owner {
        "can receive files".to_string()
    } else if !same_owner {
        "not available: different Tailscale user".to_string()
    } else if node.no_file_sharing_reason.is_empty() {
        "not available".to_string()
    } else {
        node.no_file_sharing_reason.clone()
    };
    push_detail(&mut body, &mut rows, "Taildrop", taildrop);
    push_detail(
        &mut body,
        &mut rows,
        "Traffic",
        format!(
            "↓ {}  ↑ {}",
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

    let title = gtk::Label::new(Some(&node.display_name()));
    title.add_css_class("title-2");
    title.set_wrap(true);
    title.set_margin_top(16);
    title.set_margin_bottom(8);
    title.set_margin_start(16);
    title.set_margin_end(16);
    content.append(&title);

    let scrolled_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    for (key, value) in &rows {
        scrolled_box.append(&detail_row(key, value));
    }
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .max_content_height(360)
        .child(&scrolled_box)
        .build();
    content.append(&scrolled);

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.set_margin_top(12);
    actions.set_margin_bottom(16);
    actions.set_margin_start(16);
    actions.set_margin_end(16);

    let copy_button = gtk::Button::with_label("Copy Details");
    copy_button.set_hexpand(true);
    actions.append(&copy_button);
    let send_button = if node.can_receive_taildrop() && same_owner {
        let button = gtk::Button::with_label("Send Files");
        button.add_css_class("suggested-action");
        button.set_hexpand(true);
        actions.append(&button);
        Some(button)
    } else {
        None
    };
    let exit_button = if node.exit_node_option {
        let button = gtk::Button::with_label("Use as Exit Node");
        button.set_hexpand(true);
        actions.append(&button);
        Some(button)
    } else {
        None
    };
    content.append(&actions);
    popover.set_child(Some(&content));

    let target = node.primary_ip().map(str::to_string);
    let name = node.display_name();
    let ui_clone = ui.clone();
    let details = body.trim().to_string();
    copy_button.connect_clicked(move |_| {
        ui_clone.window.clipboard().set_text(&details);
        toast(&ui_clone, "Device details copied");
    });

    if let Some(button) = send_button {
        let ui_clone = ui.clone();
        let target = target.clone();
        let name = name.clone();
        button.connect_clicked(move |_| {
            if let Some(target) = &target {
                pick_and_send(&ui_clone, target.clone(), name.clone());
            }
        });
    }

    if let Some(button) = exit_button {
        let ui_clone = ui.clone();
        let target = target.clone();
        let name = name.clone();
        button.connect_clicked(move |_| {
            if let Some(target) = &target {
                set_exit_node(&ui_clone, target.clone(), name.clone());
            }
        });
    }

    popover.set_parent(parent);
    popover.popup();
}

fn push_detail<V>(body: &mut String, rows: &mut Vec<(String, String)>, key: &str, value: V)
where
    V: Into<String>,
{
    let value = value.into();
    body.push_str(&format!("{key}: {value}\n"));
    rows.push((key.to_string(), value));
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

fn show_overview(ui: &Rc<Ui>) {
    let Some(status) = ui.last_status.borrow().clone() else {
        show_error(ui, "Status has not loaded yet");
        return;
    };
    let mut body = String::new();
    let mut rows: Vec<(String, String)> = Vec::new();

    push_detail(&mut body, &mut rows, "State", status.backend_state.label());
    if let Some(node) = &status.this_node {
        push_detail(&mut body, &mut rows, "Device", node.display_name());
        if let Some(ip) = node.primary_ip() {
            push_detail(&mut body, &mut rows, "Tailscale IP", ip);
        }
        if !node.clean_dns_name().is_empty() {
            push_detail(&mut body, &mut rows, "DNS", node.clean_dns_name());
        }
        if let Some(owner) = status.owner_label(node) {
            push_detail(&mut body, &mut rows, "Signed in as", owner);
        }
    }
    if let Some(tailnet) = &status.current_tailnet {
        if !tailnet.name.is_empty() {
            push_detail(&mut body, &mut rows, "Tailnet", tailnet.name.clone());
        }
        if !tailnet.magic_dns_suffix.is_empty() {
            push_detail(
                &mut body,
                &mut rows,
                "MagicDNS",
                tailnet.magic_dns_suffix.clone(),
            );
        }
        push_detail(
            &mut body,
            &mut rows,
            "MagicDNS enabled",
            if tailnet.magic_dns_enabled {
                "yes"
            } else {
                "no"
            },
        );
    } else if !status.magic_dns_suffix.is_empty() {
        push_detail(
            &mut body,
            &mut rows,
            "MagicDNS",
            status.magic_dns_suffix.clone(),
        );
    }
    let version = if status.version.is_empty() {
        status.client_version
    } else {
        status.version
    };
    push_detail(
        &mut body,
        &mut rows,
        "Tailscale version",
        if version.is_empty() {
            "unknown".to_string()
        } else {
            version
        },
    );
    push_detail(
        &mut body,
        &mut rows,
        "Health",
        if status.health.is_empty() {
            "OK".to_string()
        } else {
            status.health.join("\n")
        },
    );

    dialogs::show_copyable(&ui.window, "TailScout Overview", body.trim());
}

fn show_exit_node_dialog(ui: &Rc<Ui>) {
    let ui = ui.clone();
    glib::spawn_future_local(async move {
        let result = gio::spawn_blocking(tailscale::fetch_status).await;
        match result {
            Ok(Ok(status)) => {
                let options = status.exit_node_options();
                let dialog = adw::AlertDialog::new(
                    Some("Exit nodes"),
                    Some("An exit node routes your regular internet traffic through a Tailscale device. Use this like a VPN when travelling or testing from another network."),
                );
                dialog.add_response("close", "Close");
                dialog.add_response("clear", "Clear Exit Node");
                for (index, node) in options.iter().enumerate() {
                    let response = format!("use-{index}");
                    dialog.add_response(&response, &format!("Use {}", node.display_name()));
                }
                dialog.set_default_response(Some("close"));

                let ui_clone = ui.clone();
                dialog.connect_response(None, move |_, response| {
                    if response == "clear" {
                        clear_exit_node(&ui_clone);
                        return;
                    }
                    if let Some(index) = response
                        .strip_prefix("use-")
                        .and_then(|n| n.parse::<usize>().ok())
                    {
                        if let Some(node) = options.get(index) {
                            if let Some(target) = node.primary_ip() {
                                set_exit_node(&ui_clone, target.to_string(), node.display_name());
                            }
                        }
                    }
                });
                dialog.present(Some(&ui.window));
            }
            Ok(Err(err)) => handle_backend_error(&ui, &err),
            Err(_) => show_error(&ui, "Could not load exit node information"),
        }
    });
}

fn set_exit_node(ui: &Rc<Ui>, target: String, name: String) {
    run_action(ui, format!("Using {name} as exit node"), move || {
        tailscale::set_exit_node(&target)
    });
}

fn clear_exit_node(ui: &Rc<Ui>) {
    run_action(
        ui,
        "Exit node cleared".to_string(),
        tailscale::clear_exit_node,
    );
}

fn show_advertise_exit_node_dialog(ui: &Rc<Ui>) {
    let dialog = adw::AlertDialog::new(
        Some("Offer this device as an exit node?"),
        Some("This advertises that this computer can route internet traffic for your tailnet. It still needs admin approval in Tailscale before other devices can use it."),
    );
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("disable", "Disable");
    dialog.add_response("enable", "Enable");
    dialog.set_response_appearance("enable", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("cancel"));

    let ui_for_response = ui.clone();
    dialog.connect_response(None, move |_, response| match response {
        "enable" => run_action(
            &ui_for_response,
            "This device is advertising exit-node capability".to_string(),
            || tailscale::advertise_exit_node(true),
        ),
        "disable" => run_action(
            &ui_for_response,
            "Exit-node advertising disabled".to_string(),
            || tailscale::advertise_exit_node(false),
        ),
        _ => {}
    });
    dialog.present(Some(&ui.window));
}

fn setup_operator(ui: &Rc<Ui>) {
    let command = operator_command();
    let dialog = adw::AlertDialog::new(
        Some("Allow TailScout to control Tailscale?"),
        Some(&format!(
            "Tailscale actions need either root or operator permission. TailScout can open the system password prompt once and run:\n\npkexec {command}\n\nAfter this, connect/disconnect, account switching, exit nodes, and Taildrop actions should work without starting TailScout with sudo."
        )),
    );
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("setup", "Open Password Prompt");
    dialog.set_response_appearance("setup", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("setup"));

    let ui_for_response = ui.clone();
    dialog.connect_response(None, move |_, response| {
        if response == "setup" {
            run_action(
                &ui_for_response,
                "Operator permission configured".to_string(),
                tailscale::set_operator_to_current_user,
            );
        }
    });
    dialog.present(Some(&ui.window));
}

fn login_account(ui: &Rc<Ui>) {
    run_output_action(ui, "Tailscale Login", tailscale::login);
}

fn confirm_logout(ui: &Rc<Ui>) {
    let dialog = adw::AlertDialog::new(
        Some("Logout from this Tailscale account?"),
        Some("This disconnects Tailscale and expires this machine's node key. You will need to log in again later."),
    );
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("logout", "Logout");
    dialog.set_response_appearance("logout", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));

    let ui_for_response = ui.clone();
    dialog.connect_response(None, move |_, response| {
        if response == "logout" {
            run_action(
                &ui_for_response,
                "Logged out of Tailscale".to_string(),
                tailscale::logout,
            );
        }
    });
    dialog.present(Some(&ui.window));
}

fn show_profiles(ui: &Rc<Ui>) {
    let ui = ui.clone();
    glib::spawn_future_local(async move {
        let result = gio::spawn_blocking(tailscale::profiles).await;
        match result {
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
            "TailScout is connected to Tailscale, but this Linux user is not allowed to read saved Tailscale profiles yet.\n\nClick Fix Permission to open the system password prompt once. You can still login or logout from here.\n\n{detail}"
        )),
    );
    dialog.add_response("close", "Close");
    dialog.add_response("login", "Login");
    dialog.add_response("logout", "Logout");
    dialog.add_response("fix", "Fix Permission");
    dialog.set_default_response(Some("close"));

    let ui_for_response = ui.clone();
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
        "No saved Tailscale accounts found.".to_string()
    } else {
        profiles
            .iter()
            .map(|profile| {
                format!(
                    "{}{} · {}",
                    if profile.selected { "✓ " } else { "" },
                    profile.display_name(),
                    profile.tailnet
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let dialog = adw::AlertDialog::new(Some("Accounts and tailnets"), Some(&body));
    dialog.add_response("close", "Close");
    dialog.add_response("login", "Add / Login Account");
    dialog.add_response("logout", "Logout Current");
    for (index, profile) in profiles.iter().enumerate() {
        if !profile.selected {
            dialog.add_response(
                &format!("switch-{index}"),
                &format!("Switch to {}", profile.display_name()),
            );
        }
    }
    dialog.set_default_response(Some("close"));

    let ui_clone = ui.clone();
    dialog.connect_response(None, move |_, response| {
        if response == "login" {
            login_account(&ui_clone);
            return;
        }
        if response == "logout" {
            confirm_logout(&ui_clone);
            return;
        }
        if let Some(index) = response
            .strip_prefix("switch-")
            .and_then(|n| n.parse::<usize>().ok())
        {
            if let Some(profile) = profiles.get(index) {
                let id = if profile.id.is_empty() {
                    profile.display_name()
                } else {
                    profile.id.clone()
                };
                let name = profile.display_name();
                run_action(&ui_clone, format!("Switched to {name}"), move || {
                    tailscale::switch_profile(&id)
                });
            }
        }
    });
    dialog.present(Some(&ui.window));
}

fn receive_into_default_or_pick(ui: &Rc<Ui>) {
    let default_dir = ui.settings.borrow().taildrop_dir.clone();
    if let Some(directory) = default_dir.filter(|path| path.is_dir()) {
        receive_files(ui, directory);
        return;
    }
    pick_receive_directory(ui);
}

fn update_taildrop_folder_row(ui: &Rc<Ui>) {
    let settings = ui.settings.borrow();
    let subtitle = settings
        .taildrop_dir
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "Not set · click to choose in Preferences".to_string());
    ui.taildrop_folder_row.set_subtitle(&subtitle);
    ui.receive_button.set_tooltip_text(Some(&format!(
        "Receive Taildrop files{}",
        settings
            .taildrop_dir
            .as_ref()
            .map(|path| format!(" into {}", path.display()))
            .unwrap_or_default()
    )));
}

fn show_preferences(ui: &Rc<Ui>) {
    let current = ui
        .settings
        .borrow()
        .taildrop_dir
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "Not set".to_string());
    let dialog = adw::AlertDialog::new(
        Some("Preferences"),
        Some(&format!(
            "Default Taildrop receive folder:\n{current}\n\nWhen set, the header download button receives files there immediately. You can still change it anytime."
        )),
    );
    dialog.add_response("close", "Close");
    dialog.add_response("choose", "Choose Folder");
    dialog.add_response("clear", "Clear Default");
    dialog.add_response("receive", "Receive Now");
    dialog.set_default_response(Some("close"));

    let ui_for_response = ui.clone();
    dialog.connect_response(None, move |_, response| match response {
        "choose" => pick_default_taildrop_directory(&ui_for_response),
        "clear" => clear_default_taildrop_directory(&ui_for_response),
        "receive" => receive_into_default_or_pick(&ui_for_response),
        _ => {}
    });
    dialog.present(Some(&ui.window));
}

fn pick_default_taildrop_directory(ui: &Rc<Ui>) {
    let dialog = gtk::FileDialog::builder()
        .title("Choose default Taildrop receive folder")
        .modal(true)
        .build();

    let ui = ui.clone();
    let window = ui.window.clone();
    dialog.select_folder(Some(&window), gio::Cancellable::NONE, move |result| {
        let file = match result {
            Ok(file) => file,
            Err(_) => return,
        };
        let Some(path) = file.path() else {
            show_error(&ui, "Choose a local folder");
            return;
        };
        if let Err(err) = settings::save_taildrop_dir(&path) {
            show_copyable_error(&ui, "Could not save preference", &err.to_string());
            return;
        }
        ui.settings.replace(settings::load());
        update_taildrop_folder_row(&ui);
        toast(&ui, "Default Taildrop folder saved");
    });
}

fn clear_default_taildrop_directory(ui: &Rc<Ui>) {
    if let Err(err) = settings::clear_taildrop_dir() {
        show_copyable_error(ui, "Could not clear preference", &err.to_string());
        return;
    }
    ui.settings.replace(settings::load());
    update_taildrop_folder_row(ui);
    toast(ui, "Default Taildrop folder cleared");
}

fn pick_receive_directory(ui: &Rc<Ui>) {
    let dialog = gtk::FileDialog::builder()
        .title("Receive Taildrop files into folder")
        .modal(true)
        .build();

    let ui = ui.clone();
    let window = ui.window.clone();
    dialog.select_folder(Some(&window), gio::Cancellable::NONE, move |result| {
        let file = match result {
            Ok(file) => file,
            Err(_) => return,
        };
        let Some(path) = file.path() else {
            show_error(&ui, "Choose a local folder");
            return;
        };
        receive_files(&ui, path);
    });
}

fn receive_files(ui: &Rc<Ui>, directory: PathBuf) {
    run_action(
        ui,
        format!("Received Taildrop files into {}", directory.display()),
        move || tailscale::receive_files(&directory),
    );
}

fn pick_and_send(ui: &Rc<Ui>, target: String, device_name: String) {
    let dialog = gtk::FileDialog::builder()
        .title(format!("Send files to {device_name}"))
        .modal(true)
        .build();

    let ui = ui.clone();
    let window = ui.window.clone();
    dialog.open_multiple(Some(&window), gio::Cancellable::NONE, move |result| {
        let model = match result {
            Ok(model) => model,
            Err(_) => return,
        };

        let mut paths: Vec<PathBuf> = Vec::new();
        for index in 0..model.n_items() {
            if let Some(object) = model.item(index) {
                if let Ok(file) = object.downcast::<gio::File>() {
                    if let Some(path) = file.path() {
                        paths.push(path);
                    }
                }
            }
        }

        if !paths.is_empty() {
            send_files(&ui, paths, target.clone(), device_name.clone());
        }
    });
}

fn send_files(ui: &Rc<Ui>, paths: Vec<PathBuf>, target: String, device_name: String) {
    if ui.busy.get() {
        return;
    }
    set_busy(ui, true);
    let count = paths.len();

    let ui = ui.clone();
    glib::spawn_future_local(async move {
        let send_target = target.clone();
        let result = gio::spawn_blocking(move || {
            let mut errors: Vec<String> = Vec::new();
            for path in &paths {
                if let Err(err) = tailscale::send_file(path, &send_target) {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.display().to_string());
                    errors.push(format!("{name}: {err}"));
                }
            }
            errors
        })
        .await;

        set_busy(&ui, false);
        match result {
            Ok(errors) if errors.is_empty() => {
                toast(&ui, &format!("Sent {count} file(s) to {device_name}"));
            }
            Ok(errors)
                if errors
                    .iter()
                    .any(|error| is_taildrop_different_user_message(error)) =>
            {
                show_copyable_error(
                    &ui,
                    "Taildrop cannot send to this device",
                    &format!(
                        "Tailscale refused this Taildrop transfer because {device_name} is owned by a different Tailscale user.\n\nTaildrop send currently only works between devices owned by the same account/user. Send to one of your own devices, or use another file-transfer method for this peer.\n\nOriginal error:\n{}",
                        errors.join("\n")
                    ),
                )
            }
            Ok(errors) => show_copyable_error(&ui, "Some Taildrop files failed", &errors.join("\n")),
            Err(_) => show_copyable_error(&ui, "Send failed", "Background task error"),
        }
    });
}

fn is_taildrop_different_user_message(message: &str) -> bool {
    let message = message.to_lowercase();
    message.contains("cannot send files") && message.contains("different user")
}

fn run_action<F>(ui: &Rc<Ui>, success_message: String, action: F)
where
    F: FnOnce() -> tailscale::Result<()> + Send + 'static,
{
    if ui.busy.get() {
        return;
    }
    set_busy(ui, true);

    let ui = ui.clone();
    glib::spawn_future_local(async move {
        let result = gio::spawn_blocking(action).await;
        set_busy(&ui, false);
        match result {
            Ok(Ok(())) => toast(&ui, &success_message),
            Ok(Err(err)) => handle_action_error(&ui, &err),
            Err(_) => show_copyable_error(&ui, "Action failed", "Background task error"),
        }
        refresh(&ui);
    });
}

fn run_output_action<F>(ui: &Rc<Ui>, title: &str, action: F)
where
    F: FnOnce() -> tailscale::Result<String> + Send + 'static,
{
    if ui.busy.get() {
        return;
    }
    set_busy(ui, true);

    let title = title.to_string();
    let ui = ui.clone();
    glib::spawn_future_local(async move {
        let result = gio::spawn_blocking(action).await;
        set_busy(&ui, false);
        match result {
            Ok(Ok(output)) => {
                let body = if output.trim().is_empty() {
                    "Command completed successfully.".to_string()
                } else {
                    output
                };
                dialogs::show_copyable(&ui.window, &title, &body);
            }
            Ok(Err(err)) => handle_action_error(&ui, &err),
            Err(_) => show_copyable_error(&ui, "Command failed", "Background task error"),
        }
        refresh(&ui);
    });
}

fn set_busy(ui: &Rc<Ui>, busy: bool) {
    ui.busy.set(busy);
    ui.spinner.set_visible(busy);
    if busy {
        ui.spinner.start();
    } else {
        ui.spinner.stop();
    }
    ui.refresh_button.set_sensitive(!busy);
    ui.connect_button.set_sensitive(!busy);
    ui.setup_button.set_sensitive(!busy);
    ui.receive_button.set_sensitive(!busy);
    ui.admin_button.set_sensitive(!busy);
    ui.profiles_button.set_sensitive(!busy);
    ui.help_button.set_sensitive(!busy);
}

fn update_connect_button(ui: &Rc<Ui>, state: &BackendState) {
    let (label, add, remove) = match state {
        BackendState::NeedsLogin => ("Log In", "suggested-action", "destructive-action"),
        BackendState::Running => ("Disconnect", "destructive-action", "suggested-action"),
        _ => ("Connect", "suggested-action", "destructive-action"),
    };
    ui.connect_button.set_label(label);
    ui.connect_button.remove_css_class(remove);
    ui.connect_button.add_css_class(add);
}

fn set_state_pill(ui: &Rc<Ui>, state: &BackendState, running: bool) {
    ui.state_label.set_text(&state.label());
    ui.state_label.remove_css_class("online");
    ui.state_label.remove_css_class("offline");
    ui.state_label
        .add_css_class(if running { "online" } else { "offline" });
}

fn handle_backend_error(ui: &Rc<Ui>, err: &TailscaleError) {
    if err.is_permission_problem() {
        show_permission_dialog(ui, &err.to_string());
    } else {
        show_error(ui, &err.to_string());
    }
}

fn handle_action_error(ui: &Rc<Ui>, err: &TailscaleError) {
    if err.is_permission_problem() {
        show_permission_dialog(ui, &err.to_string());
    } else if err.is_taildrop_different_user_problem() {
        show_copyable_error(
            ui,
            "Taildrop cannot send to this device",
            &format!(
                "Tailscale refused this Taildrop transfer because the peer is owned by a different Tailscale user.\n\nTaildrop send currently only works between devices owned by the same account/user. Use another file-transfer method, or send to a device owned by your current Tailscale user.\n\nOriginal error:\n{err}"
            ),
        );
    } else {
        show_copyable_error(ui, "Tailscale command failed", &err.to_string());
    }
}

fn show_permission_dialog(ui: &Rc<Ui>, detail: &str) {
    let command = operator_command();
    show_error(
        ui,
        "Permission needed. TailScout can open the system password prompt to fix operator access.",
    );
    let dialog = adw::AlertDialog::new(
        Some("Permission needed"),
        Some(&format!(
            "You are logged in to Tailscale, but this Linux user is not allowed to control tailscaled directly.\n\nClick Fix Permission to open the system password prompt and run:\n\npkexec {command}\n\nOr copy this command and run it manually with sudo:\n\nsudo {command}\n\nOriginal error:\n{detail}"
        )),
    );
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("details", "Details");
    dialog.add_response("copy", "Copy Command");
    dialog.add_response("login", "Try Login");
    dialog.add_response("admin", "Admin Console");
    dialog.add_response("setup", "Fix Permission");
    dialog.set_response_appearance("setup", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("setup"));

    let ui_for_response = ui.clone();
    let detail = detail.to_string();
    dialog.connect_response(None, move |_, response| match response {
        "setup" => setup_operator(&ui_for_response),
        "copy" => {
            let command = format!("sudo {}", operator_command());
            ui_for_response.window.clipboard().set_text(&command);
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
    let user = env::var("USER").unwrap_or_else(|_| "$USER".to_string());
    format!("tailscale set --operator={user}")
}

fn show_copyable_error(ui: &Rc<Ui>, title: &str, body: &str) {
    show_error(ui, title);
    dialogs::show_copyable(&ui.window, title, body);
}

fn show_login_output(ui: &Rc<Ui>, output: &str) {
    if let Some(url) = dialogs::first_url(output) {
        dialogs::open_uri(&ui.window, &url);
        toast(ui, "Login link opened in browser");
    }
    dialogs::show_copyable(&ui.window, "Tailscale Login", output);
}

fn show_guide(ui: &Rc<Ui>) {
    dialogs::show_copyable(&ui.window, "TailScout Guide", GUIDE);
}

fn show_cli_cheatsheet(ui: &Rc<Ui>) {
    dialogs::show_copyable(&ui.window, "Tailscale CLI Cheatsheet", CLI_CHEATSHEET);
}

fn show_about(ui: &Rc<Ui>) {
    let dialog = adw::AboutDialog::builder()
        .application_name("TailScout")
        .application_icon("dev.shre.TailScout")
        .developer_name("Shreyam Adhikari")
        .version("0.1.0")
        .website("https://shreyam1008.com.np")
        .issue_url("https://github.com/shreyam1008/tailScout/issues")
        .copyright("© 2026 Shreyam Adhikari — MIT License")
        .comments("Native Rust + GTK4/libadwaita GUI for Tailscale on Linux.")
        .build();
    dialog.present(Some(&ui.window));
}

fn show_error(ui: &Rc<Ui>, message: &str) {
    ui.banner.set_title(message);
    ui.banner.add_css_class("error");
    ui.banner.set_revealed(true);
}

fn show_hint(ui: &Rc<Ui>, message: &str) {
    ui.banner.set_title(message);
    ui.banner.remove_css_class("error");
    ui.banner.set_revealed(true);
}

fn toast(ui: &Rc<Ui>, message: &str) {
    ui.toasts.add_toast(adw::Toast::new(message));
}
