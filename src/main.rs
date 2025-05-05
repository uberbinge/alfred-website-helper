use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Deserialize)]
struct Site {
    title: String,
    arg: String,
    icon: Option<String>,
}

#[derive(Serialize)]
struct AlfredItem {
    uid: String,
    title: String,
    subtitle: String,
    arg: String,
    valid: bool,
    icon: AlfredIcon, // Fixed: Use `AlfredIcon` instead of `Icon`
}

#[derive(Serialize)]
struct AlfredIcon {
    path: String,
}

#[derive(Serialize)]
struct AlfredOutput {
    items: Vec<AlfredItem>,
}

fn main() {
    // Get the query from command-line arguments
    let args: Vec<String> = env::args().collect();
    let query = args.get(1).map(|s| s.to_lowercase()).unwrap_or_default();

    // Read sites.json
    let config_path = env::var("HOME").unwrap_or_default() + "/.config/alfred/sites.json";
    let sites: Vec<Site> = match fs::read_to_string(&config_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(sites) => sites,
            Err(_) => {
                println!(
                    r#"{{"items": [{{"title": "Error", "subtitle": "Invalid JSON in sites.json", "valid": false}}]}}"#
                );
                return;
            }
        },
        Err(_) => {
            println!(
                r#"{{"items": [{{"title": "Error", "subtitle": "Could not read {}", "valid": false}}]}}"#,
                config_path
            );
            return;
        }
    };

    // Filter sites and create Alfred items
    let default_icon = format!("{}/.config/alfred/default.png", env::var("HOME").unwrap_or_default());
    let items: Vec<AlfredItem> = sites
        .into_iter()
        .filter(|site| query.is_empty() || site.title.to_lowercase().contains(&query))
        .map(|site| AlfredItem {
            uid: site.title.clone(),
            title: site.title.clone(),
            subtitle: site.arg.clone(),
            arg: site.arg,
            valid: true,
            icon: AlfredIcon {
                path: site.icon.unwrap_or(default_icon.clone()),
            },
        })
        .collect();

    // Output Alfred JSON
    let output = AlfredOutput { items };
    println!("{}", serde_json::to_string(&output).unwrap_or_else(|_| {
        r#"{"items": [{"title": "Error", "subtitle": "Failed to serialize JSON", "valid": false}]}"#.to_string()
    }));
}
