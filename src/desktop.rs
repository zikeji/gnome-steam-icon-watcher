use log::{warn};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub fn icon_exists(app_id: &str) -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    let hicolor_path = format!("{}/.local/share/icons/hicolor", home);
    if let Ok(entries) = fs::read_dir(&hicolor_path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let apps_path = entry.path().join("apps");
                    if apps_path.is_dir() {
                        let icon_path = apps_path.join(format!("steam_icon_{}.png", app_id));
                        if icon_path.exists() {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn get_desktop_path(game_name: &str) -> String {
    format!(
        "{}/.local/share/applications/{}.desktop",
        std::env::var("HOME").unwrap_or_default(),
        game_name
    )
}

pub fn write_desktop_file(game_name: &str, app_id: &str) {
    let desktop_path = get_desktop_path(game_name);
    let icon_name = if icon_exists(app_id) {
        format!("steam_icon_{}", app_id)
    } else {
        warn!("No Steam icon found for '{}'. To get the correct icon, use 'Manage → Add Desktop Shortcut' in Steam for this game.", game_name);
        "steam".to_string()
    };
    if !Path::new(&desktop_path).exists() {
        let desktop_contents = format!(
            "[Desktop Entry]\nName={}\nExec=steam steam://rungameid/{}\nType=Application\nIcon={}\nCategories=Game;\nTerminal=false\nStartupWMClass=steam_app_{}\nComment=Play {} on Steam\n",
            game_name, app_id, icon_name, app_id, game_name
        );
        if let Err(e) = fs::write(&desktop_path, desktop_contents) {
            let _ = writeln!(io::stderr(), "Failed to write desktop file: {}", e);
        }
    }
}

pub fn remove_desktop_file(game_name: &str) {
    let desktop_path = get_desktop_path(game_name);
    if let Err(e) = fs::remove_file(&desktop_path) {
        let _ = writeln!(io::stderr(), "Failed to remove desktop file: {}", e);
    }
}
