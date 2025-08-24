use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::process::Command;

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

fn resolve_onepassword_url(url: &str) -> String {
    if url.starts_with("op://") {
        match Command::new("op")
            .args(&["read", url, "--account=my.1password.eu"])
            .output()
        {
            Ok(output) if output.status.success() => {
                String::from_utf8(output.stdout)
                    .unwrap_or_else(|_| url.to_string())
                    .trim()
                    .to_string()
            }
            _ => url.to_string(), // Fall back to original URL if resolution fails
        }
    } else {
        url.to_string()
    }
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
        .map(|site| {
            let resolved_url = resolve_onepassword_url(&site.arg);
            AlfredItem {
                uid: site.title.clone(),
                title: site.title.clone(),
                subtitle: resolved_url.clone(),
                arg: resolved_url,
                valid: true,
                icon: AlfredIcon {
                    path: site.icon.unwrap_or(default_icon.clone()),
                },
            }
        })
        .collect();

    // Output Alfred JSON
    let output = AlfredOutput { items };
    println!("{}", serde_json::to_string(&output).unwrap_or_else(|_| {
        r#"{"items": [{"title": "Error", "subtitle": "Failed to serialize JSON", "valid": false}]}"#.to_string()
    }));
}
