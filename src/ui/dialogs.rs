use adw::prelude::*;
use gtk::gio;

pub fn show_copyable<P>(parent: &P, title: &str, body: &str)
where
    P: IsA<gtk::Window> + IsA<gtk::Widget>,
{
    let window = gtk::Window::builder()
        .title(title)
        .modal(true)
        .transient_for(parent)
        .default_width(560)
        .default_height(420)
        .build();

    let header = adw::HeaderBar::new();
    let copy_button = gtk::Button::with_label("Copy");
    copy_button.add_css_class("suggested-action");
    header.pack_end(&copy_button);

    let buffer = gtk::TextBuffer::new(None);
    buffer.set_text(body);

    let text = gtk::TextView::with_buffer(&buffer);
    text.set_editable(false);
    text.set_cursor_visible(false);
    text.set_monospace(true);
    text.set_wrap_mode(gtk::WrapMode::WordChar);
    text.add_css_class("copyable-output");

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
    window.set_child(Some(&body_box));

    let body = body.to_string();
    copy_button.connect_clicked(move |button| {
        button.clipboard().set_text(&body);
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
