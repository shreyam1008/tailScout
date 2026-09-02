use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};

use crate::{settings, tailscale};

use super::actions::{run_action, set_busy, show_copyable_error, show_error, toast};
use super::Ui;

pub(super) fn receive_into_default_or_pick(ui: &Rc<Ui>) {
    match ui
        .settings
        .borrow()
        .taildrop_dir
        .clone()
        .filter(|path| path.is_dir())
    {
        Some(directory) => receive_files(ui, directory),
        None => pick_receive_directory(ui),
    }
}

pub(super) fn update_taildrop_folder_row(ui: &Rc<Ui>) {
    let settings = ui.settings.borrow();
    let directory = settings
        .taildrop_dir
        .as_ref()
        .map(|path| path.display().to_string());
    ui.taildrop_folder_row.set_subtitle(
        directory
            .as_deref()
            .unwrap_or("Not set · click to choose in Preferences"),
    );
    ui.receive_button.set_tooltip_text(Some(&match directory {
        Some(path) => format!("Receive Taildrop files into {path}"),
        None => "Receive Taildrop files".into(),
    }));
}

pub(super) fn show_preferences(ui: &Rc<Ui>) {
    let current = ui
        .settings
        .borrow()
        .taildrop_dir
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "Not set".into());
    let dialog = adw::AlertDialog::new(
        Some("Preferences"),
        Some(&format!(
            "Default Taildrop receive folder:\n{current}\n\nWhen set, the header download button receives files there immediately."
        )),
    );
    for (id, label) in [
        ("close", "Close"),
        ("choose", "Choose Folder"),
        ("clear", "Clear Default"),
        ("receive", "Receive Now"),
    ] {
        dialog.add_response(id, label);
    }
    dialog.set_default_response(Some("close"));

    let ui_for_response = Rc::clone(ui);
    dialog.connect_response(None, move |_, response| match response {
        "choose" => pick_default_taildrop_directory(&ui_for_response),
        "clear" => clear_default_taildrop_directory(&ui_for_response),
        "receive" => receive_into_default_or_pick(&ui_for_response),
        _ => {}
    });
    dialog.present(Some(&ui.window));
}

fn pick_default_taildrop_directory(ui: &Rc<Ui>) {
    pick_directory(ui, "Choose default Taildrop receive folder", |ui, path| {
        if let Err(err) = settings::save_taildrop_dir(&path) {
            show_copyable_error(ui, "Could not save preference", &err.to_string());
            return;
        }
        ui.settings.replace(settings::load());
        update_taildrop_folder_row(ui);
        toast(ui, "Default Taildrop folder saved");
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
    pick_directory(ui, "Receive Taildrop files into folder", receive_files);
}

fn pick_directory<F>(ui: &Rc<Ui>, title: &str, on_selected: F)
where
    F: FnOnce(&Rc<Ui>, PathBuf) + 'static,
{
    if !begin_file_dialog(ui) {
        return;
    }
    let dialog = gtk::FileDialog::builder().title(title).modal(true).build();
    let ui = Rc::clone(ui);
    let window = ui.window.clone();
    dialog.select_folder(Some(&window), gio::Cancellable::NONE, move |result| {
        end_file_dialog(&ui);
        let Some(path) = result.ok().and_then(|file| file.path()) else {
            return;
        };
        on_selected(&ui, path);
    });
}

fn receive_files(ui: &Rc<Ui>, directory: PathBuf) {
    run_action(
        ui,
        format!("Received Taildrop files into {}", directory.display()),
        move || tailscale::receive_files(&directory),
    );
}

pub(super) fn pick_and_send(ui: &Rc<Ui>, target: String, device_name: String) {
    if !begin_file_dialog(ui) {
        return;
    }
    let dialog = gtk::FileDialog::builder()
        .title(format!("Send files to {device_name}"))
        .modal(true)
        .build();
    let ui = Rc::clone(ui);
    let window = ui.window.clone();
    dialog.open_multiple(Some(&window), gio::Cancellable::NONE, move |result| {
        end_file_dialog(&ui);
        let Some(model) = result.ok() else {
            return;
        };
        let paths = (0..model.n_items())
            .filter_map(|index| model.item(index))
            .filter_map(|object| object.downcast::<gio::File>().ok())
            .filter_map(|file| file.path())
            .collect::<Vec<_>>();
        if !paths.is_empty() {
            send_files(&ui, paths, target.clone(), device_name.clone());
        }
    });
}

fn begin_file_dialog(ui: &Rc<Ui>) -> bool {
    if ui.file_dialog_open.replace(true) {
        toast(ui, "A file chooser is already open");
        return false;
    }
    true
}

fn end_file_dialog(ui: &Rc<Ui>) {
    ui.file_dialog_open.set(false);
}

fn send_files(ui: &Rc<Ui>, paths: Vec<PathBuf>, target: String, device_name: String) {
    if ui.busy.get() {
        return;
    }
    set_busy(ui, true);
    let count = paths.len();
    let ui = Rc::clone(ui);
    glib::spawn_future_local(async move {
        let result = gio::spawn_blocking(move || {
            paths
                .iter()
                .filter_map(|path| {
                    tailscale::send_file(path, &target).err().map(|err| {
                        let name = path
                            .file_name()
                            .map(|value| value.to_string_lossy().into_owned())
                            .unwrap_or_else(|| path.display().to_string());
                        format!("{name}: {err}")
                    })
                })
                .collect::<Vec<_>>()
        })
        .await;

        set_busy(&ui, false);
        match result {
            Ok(errors) if errors.is_empty() => {
                toast(&ui, &format!("Sent {count} file(s) to {device_name}"));
            }
            Ok(errors) if errors.iter().any(|error| is_different_user(error)) => {
                show_copyable_error(
                    &ui,
                    "Taildrop cannot send to this device",
                    &format!(
                        "{device_name} belongs to a different Tailscale user. Use another transfer method or one of your own devices.\n\nOriginal error:\n{}",
                        errors.join("\n")
                    ),
                );
            }
            Ok(errors) => {
                show_copyable_error(&ui, "Some Taildrop files failed", &errors.join("\n"));
            }
            Err(_) => show_error(&ui, "Taildrop background task failed"),
        }
    });
}

fn is_different_user(message: &str) -> bool {
    let message = message.to_lowercase();
    message.contains("cannot send files") && message.contains("different user")
}
