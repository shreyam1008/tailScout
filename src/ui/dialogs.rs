use adw::prelude::*;
use gtk::{gio, glib};

pub fn show_copyable<P>(parent: &P, title: &str, body: &str)
where
    P: IsA<gtk::Window> + IsA<gtk::Widget>,
{
    let window = gtk::Window::builder()
        .title(title)
        .modal(true)
        .transient_for(parent)
        .default_width(640)
        .default_height(460)
        .build();

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&adw::WindowTitle::new(title, "Copyable output")));
    let copy_button = gtk::Button::with_label("Copy All");
    copy_button.add_css_class("suggested-action");
    copy_button.set_tooltip_text(Some("Copy all text to clipboard"));
    header.pack_end(&copy_button);

    let buffer = gtk::TextBuffer::new(None);
    buffer.set_text(body);

    let text = gtk::TextView::with_buffer(&buffer);
    text.set_editable(false);
    text.set_cursor_visible(false);
    text.set_monospace(true);
    text.set_wrap_mode(gtk::WrapMode::WordChar);
    text.add_css_class("copyable-output");
    text.set_left_margin(14);
    text.set_right_margin(14);
    text.set_top_margin(14);
    text.set_bottom_margin(14);

    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .child(&text)
        .build();

    let body_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    body_box.append(&header);
    body_box.append(&scrolled);
    body_box.add_css_class("copyable-window");
    window.set_child(Some(&body_box));

    let body = body.to_string();
    let copy_button_for_reset = copy_button.clone();
    copy_button.connect_clicked(move |button| {
        button.clipboard().set_text(&body);
        button.set_label("Copied");
        let copy_button_for_reset = copy_button_for_reset.clone();
        glib::timeout_add_seconds_local_once(2, move || {
            copy_button_for_reset.set_label("Copy All");
        });
    });

    window.present();
}

pub fn open_uri<P>(parent: &P, uri: &str)
where
    P: IsA<gtk::Window> + IsA<gtk::Widget>,
{
    if let Err(err) = gio::AppInfo::launch_default_for_uri(uri, None::<&gio::AppLaunchContext>) {
        show_copyable(parent, "Could not open link", &format!("{uri}\n\n{err}"));
    }
}

pub fn first_url(input: &str) -> Option<String> {
    input.split_whitespace().find_map(|part| {
        let trimmed = part.trim_matches(|ch: char| {
            matches!(ch, '<' | '>' | '(' | ')' | '[' | ']' | ',' | '.' | ';')
        });
        (trimmed.starts_with("https://") || trimmed.starts_with("http://"))
            .then(|| trimmed.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::first_url;

    #[test]
    fn extracts_first_url() {
        assert_eq!(
            first_url("open https://login.tailscale.com/a/abc now"),
            Some("https://login.tailscale.com/a/abc".to_string())
        );
    }

    #[test]
    fn ignores_missing_url() {
        assert_eq!(first_url("no url here"), None);
    }
}
