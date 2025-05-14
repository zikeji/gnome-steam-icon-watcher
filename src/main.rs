use std::{collections::HashMap, thread, time::Duration};
use env_logger::Env;
use procfs::process::all_processes;
use log::info;
use regex::Regex;
use std::fs;
use std::path::{Path};
use std::io::{self, Write};

fn icon_exists(app_id: &str) -> bool {
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

fn get_manifest_path(app_id: &str) -> String {
    format!(
        "{}/.local/share/Steam/steamapps/appmanifest_{}.acf",
        std::env::var("HOME").unwrap_or_else(|_| String::from("")),
        app_id
    )
}

fn get_desktop_path(game_name: &str) -> String {
    format!(
        "{}/.local/share/applications/{}.desktop",
        std::env::var("HOME").unwrap_or_default(),
        game_name
    )
}

fn extract_game_name(manifest_contents: &str) -> Option<String> {
    let name_re = Regex::new(r#"name"\s+"(.*?)""#).unwrap();
    name_re.captures(manifest_contents).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

fn write_desktop_file(game_name: &str, app_id: &str) {
    let desktop_path = get_desktop_path(game_name);
    if !Path::new(&desktop_path).exists() {
        let icon_name = if icon_exists(app_id) {
            format!("steam_icon_{}", app_id)
        } else {
            "steam".to_string()
        };
        let desktop_contents = format!(
            "[Desktop Entry]\nName={}\nExec=steam steam://rungameid/{}\nType=Application\nIcon={}\nCategories=Game;\nTerminal=false\nStartupWMClass=steam_app_{}\nComment=Play {} on Steam\n",
            game_name, app_id, icon_name, app_id, game_name
        );
        if let Err(e) = fs::write(&desktop_path, desktop_contents) {
            let _ = writeln!(io::stderr(), "Failed to write desktop file: {}", e);
        }
    }
}

fn remove_desktop_file(game_name: &str) {
    let desktop_path = get_desktop_path(game_name);
    if let Err(e) = fs::remove_file(&desktop_path) {
        let _ = writeln!(io::stderr(), "Failed to remove desktop file: {}", e);
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let mut active_games: HashMap<i32, (String, String)> = HashMap::new();
    loop {
        let mut current_games: HashMap<i32, (String, String)> = HashMap::new();
        if let Ok(procs) = all_processes() {
            for proc in procs {
                if let Ok(p) = proc {
                    let pid = p.pid();
                    if let Ok(cmdline) = p.cmdline() {
                        let joined = cmdline.join(" ");
                        let re = Regex::new(r"SteamLaunch AppId=(\d+)").unwrap();
                        if let Some(caps) = re.captures(&joined) {
                            if let Some(app_id) = caps.get(1) {
                                let manifest_path = get_manifest_path(app_id.as_str());
                                if let Ok(contents) = fs::read_to_string(&manifest_path) {
                                    if let Some(game_name) = extract_game_name(&contents) {
                                        let app_id = app_id.as_str().to_string();
                                        current_games.insert(pid, (game_name.clone(), app_id.clone()));
                                        if !active_games.contains_key(&pid) {
                                            info!("Game '{}' ({}) detected with PID {}", game_name, app_id, pid);
                                            write_desktop_file(&game_name, &app_id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        for (pid, (game_name, _app_id)) in active_games.iter() {
            if !current_games.contains_key(pid) {
                info!("Game '{}' exited (PID {})", game_name, pid);
                remove_desktop_file(game_name);
            }
        }
        active_games = current_games;
        thread::sleep(Duration::from_secs(2));
    }
}
