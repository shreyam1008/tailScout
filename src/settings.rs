use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Settings {
    pub taildrop_dir: Option<PathBuf>,
}

const SETTINGS_FILE: &str = "settings.conf";
const TAILDROP_DIR_KEY: &str = "taildrop_dir=";

pub fn load() -> Settings {
    let Some(path) = settings_file() else {
        return Settings::default();
    };
    let Ok(input) = fs::read_to_string(path) else {
        return Settings::default();
    };
    parse(&input)
}

pub fn save_taildrop_dir(path: &Path) -> io::Result<()> {
    save(&Settings {
        taildrop_dir: Some(path.to_path_buf()),
    })
}

pub fn clear_taildrop_dir() -> io::Result<()> {
    save(&Settings::default())
}

fn save(settings: &Settings) -> io::Result<()> {
    let path = settings_file().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "could not find user config directory",
        )
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serialize(settings))
}

fn settings_file() -> Option<PathBuf> {
    if let Some(config_home) = env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        return Some(
            PathBuf::from(config_home)
                .join("tailscout")
                .join(SETTINGS_FILE),
        );
    }
    env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(|home| {
            PathBuf::from(home)
                .join(".config")
                .join("tailscout")
                .join(SETTINGS_FILE)
        })
}

fn parse(input: &str) -> Settings {
    let taildrop_dir = input.lines().find_map(|line| {
        line.strip_prefix(TAILDROP_DIR_KEY)
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
    });
    Settings { taildrop_dir }
}

fn serialize(settings: &Settings) -> String {
    match &settings.taildrop_dir {
        Some(path) => format!("{TAILDROP_DIR_KEY}{}\n", path.display()),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_taildrop_dir() {
        let settings = parse("taildrop_dir=/tmp/taildrop\n");
        assert_eq!(settings.taildrop_dir, Some(PathBuf::from("/tmp/taildrop")));
    }

    #[test]
    fn ignores_empty_taildrop_dir() {
        let settings = parse("taildrop_dir=\n");
        assert_eq!(settings.taildrop_dir, None);
    }

    #[test]
    fn serializes_taildrop_dir() {
        let settings = Settings {
            taildrop_dir: Some(PathBuf::from("/tmp/taildrop")),
        };
        assert_eq!(serialize(&settings), "taildrop_dir=/tmp/taildrop\n");
    }
}
