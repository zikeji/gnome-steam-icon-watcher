use procfs::process::all_processes;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use log::info;

use crate::desktop::{write_desktop_file, remove_desktop_file};
use crate::steam::{get_manifest_path, extract_game_name};

pub fn for_steam_game_processes(active_games: &mut HashSet<(String, String)>) -> HashSet<(String, String)> {
    let mut current_games: HashSet<(String, String)> = HashSet::new();
    
    if let Ok(procs) = all_processes() {
        for proc in procs {
            if let Ok(p) = proc {
                if let Ok(cmdline) = p.cmdline() {
                    let joined = cmdline.join(" ");
                    let re = Regex::new(r"SteamLaunch AppId=(\d+)").unwrap();
                    if let Some(caps) = re.captures(&joined) {
                        if let Some(app_id) = caps.get(1) {
                            let manifest_path = get_manifest_path(app_id.as_str());
                            if let Ok(contents) = fs::read_to_string(&manifest_path) {
                                if let Some(game_name) = extract_game_name(&contents) {
                                    let app_id = app_id.as_str().to_string();
                                    current_games.insert((game_name.clone(), app_id.clone()));
                                    if !active_games.contains(&(game_name.clone(), app_id.clone())) {
                                        info!("Game '{}' ({}) detected", game_name, app_id);
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
    
    for (game_name, app_id) in active_games.difference(&current_games) {
        info!("Game '{}' ({}) exited", game_name, app_id);
        remove_desktop_file(game_name);
    }
    
    current_games
}
