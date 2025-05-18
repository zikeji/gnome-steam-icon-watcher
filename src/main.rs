mod appinfo_vdf;
mod desktop;
mod steam;
mod monitor;

use std::{thread, time::Duration};
use std::collections::HashSet;
use env_logger::Env;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let mut active_games: HashSet<(String, String)> = HashSet::new();

    loop {
        active_games = monitor::for_steam_game_processes(&mut active_games);

        thread::sleep(Duration::from_secs(2));
    }
}
