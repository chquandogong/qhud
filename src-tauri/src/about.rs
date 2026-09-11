//! Build-derived identity and the two links exposed by the About dialog.

use serde::Serialize;
use tauri_plugin_opener::OpenerExt;

const WEBSITE: &str = "https://chquandogong.github.io/CHENGHAO-QUAN/";
const REPOSITORY: &str = "https://github.com/chquandogong/qhud";

#[derive(Serialize)]
pub struct AppInfo {
    name: &'static str,
    version: &'static str,
    author: &'static str,
    website: &'static str,
    repository: &'static str,
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: "qhud",
        version: env!("CARGO_PKG_VERSION"),
        author: "Chenghao Quan (chquandogong)",
        website: WEBSITE,
        repository: REPOSITORY,
    }
}

fn link_target(target: &str) -> Result<&'static str, String> {
    match target {
        "author" => Ok(WEBSITE),
        "repository" => Ok(REPOSITORY),
        _ => Err("Unknown About link".into()),
    }
}

#[tauri::command]
pub fn open_about_link(app: tauri::AppHandle, target: String) -> Result<(), String> {
    app.opener()
        .open_url(link_target(&target)?, None::<&str>)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn about_links_only_accept_named_destinations() {
        assert_eq!(link_target("author").unwrap(), WEBSITE);
        assert_eq!(link_target("repository").unwrap(), REPOSITORY);
        for unknown in ["", "https://example.com", "file:///tmp/test", "AUTHOR"] {
            assert!(link_target(unknown).is_err());
        }
    }
}
