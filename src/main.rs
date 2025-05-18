mod appinfo_vdf;
mod desktop;
mod steam;
mod monitor;

use std::{thread, time::Duration};
use std::collections::HashSet;
use std::env;
use env_logger::Env;

#[cfg(not(target_os = "linux"))]
compile_error!("This applications only works on Linux.");

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "-v" || args[1] == "--version") {
        println!("linux-steam-icon-watcher v{}", env!("CARGO_PKG_VERSION"));
        return;
    }
    
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let mut active_games: HashSet<(String, String)> = HashSet::new();

    loop {
        active_games = monitor::for_steam_game_processes(&mut active_games);

        thread::sleep(Duration::from_secs(2));
    }
}
