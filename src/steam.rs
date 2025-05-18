use std::io;
use regex::Regex;

use crate::appinfo_vdf::get_clienticon_from_appinfo;

pub fn get_manifest_path(app_id: &str) -> String {
    format!(
        "{}/.local/share/Steam/steamapps/appmanifest_{}.acf",
        std::env::var("HOME").unwrap_or_else(|_| String::from("")),
        app_id
    )
}

pub fn extract_game_name(manifest_contents: &str) -> Option<String> {
    let name_re = Regex::new(r#"name"\s+"(.*?)""#).unwrap();
    name_re.captures(manifest_contents).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

pub fn get_clienticon_from_steam_appinfo(app_id: &str) -> io::Result<Option<String>> {
    let appinfo_path = std::path::PathBuf::from(format!(
        "{}/.local/share/Steam/appcache/appinfo.vdf",
        std::env::var("HOME").unwrap_or_default()
    ));
    get_clienticon_from_appinfo(app_id, &appinfo_path)
}
