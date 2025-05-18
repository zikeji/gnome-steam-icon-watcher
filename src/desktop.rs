use log::{error, warn};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use reqwest;
use image::{self};

use crate::steam::get_clienticon_from_steam_appinfo;

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
        match get_clienticon_from_steam_appinfo(app_id) {
            Ok(Some(clienticon)) => {
                if process_steam_icon(app_id, &clienticon) {
                    format!("steam_icon_{}", app_id)
                } else {
                    warn!("Failed to obtain icon for '{}'. To attempt to manually populate the correct icon, use 'Manage → Add Desktop Shortcut' in Steam for this game.", game_name);
                    "steam".to_string()
                }
            },
            Ok(None) => {
                warn!("No icon found for '{}'. To attempt to manually populate the correct icon, use 'Manage → Add Desktop Shortcut' in Steam for this game.", game_name);
                "steam".to_string()
            },
            Err(e) => {
                println!("Error: {}", e);
                warn!("No icon found for '{}'. To attempt to manually populate the correct icon, use 'Manage → Add Desktop Shortcut' in Steam for this game.", game_name);
                "steam".to_string()
            },
        }
    };
    if !Path::new(&desktop_path).exists() {
        let desktop_contents = format!(
            "[Desktop Entry]\nName={}\nExec=steam steam://rungameid/{}\nType=Application\nIcon={}\nCategories=Game;\nTerminal=false\nStartupWMClass=steam_app_{}\nComment=Play {} on Steam\nNoDisplay=true\n",
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

pub fn process_steam_icon(app_id: &str, clienticon: &str) -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    let local_ico_path = format!("{}/.local/share/Steam/steam/games/{}.ico", home, clienticon);

    if Path::new(&local_ico_path).exists() {
        // Load from local file
        match image::open(&local_ico_path) {
            Ok(img) => convert_icon(app_id, img),
            Err(e) => {
                error!("Failed to open icon file {}: {}", local_ico_path, e);
                false
            }
        }
    } else {
        // Download and load from memory
        match download_steam_icon(app_id, clienticon) {
            Ok(Some(icon_data)) => match image::load_from_memory(&icon_data) {
                Ok(img) => convert_icon(app_id, img),
                Err(e) => {
                    error!("Failed to download icon from Steam CDN: {}", e);
                    false
                }
            },
            _ => false
        }
    }
}

fn download_steam_icon(app_id: &str, clienticon: &str) -> io::Result<Option<Vec<u8>>> {
    let url = format!(
        "https://cdn.cloudflare.steamstatic.com/steamcommunity/public/images/apps/{}/{}.ico",
        app_id, clienticon
    );

    let client = reqwest::blocking::Client::new();
    let response = match client.get(&url).send() {
        Ok(resp) => {
            if !resp.status().is_success() {
                error!("Failed to download icon from {}: HTTP {}", url, resp.status());
                return Ok(None);
            }
            resp
        },
        Err(e) => {
            error!("Failed to download icon from {}: {}", url, e);
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }
    };

    match response.bytes() {
        Ok(bytes) => Ok(Some(bytes.to_vec())),
        Err(e) => {
            error!("Failed to read icon data: {}", e);
            Err(io::Error::new(io::ErrorKind::Other, e))
        }
    }
}

fn convert_icon(app_id: &str, img: image::DynamicImage) -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    let hicolor_path = format!("{}/.local/share/icons/hicolor", home);

    let icon_sizes = vec!["256x256", "128x128", "96x96", "64x64", "48x48", "32x32", "24x24", "16x16"];
    
    let mut success = false;

    for size in icon_sizes {
        let dir_path = format!("{}/{}/apps", hicolor_path, size);

        let dimensions: Vec<&str> = size.split('x').collect();
        if dimensions.len() != 2 {
            continue;
        }

        let width: u32 = dimensions[0].parse().unwrap_or(0);
        let height: u32 = dimensions[1].parse().unwrap_or(0);

        if width == 0 || height == 0 || img.width() < width || img.height() < height {
            continue;
        }

        let _ = fs::create_dir_all(&dir_path);
        let target_path = format!("{}/steam_icon_{}.png", dir_path, app_id);
        let resized = img.resize(width, height, image::imageops::FilterType::Lanczos3);

        if let Err(e) = resized.save_with_format(&target_path, image::ImageFormat::Png) {
            warn!("Failed to save icon to {}: {}", target_path, e);
        } else {
            success = true;
        }
    }

    if success {
        let _ = Command::new("gtk-update-icon-cache")
            .arg("--ignore-theme-index")
            .arg("-f")
            .arg(&hicolor_path)
            .output();
    }

    success
}
