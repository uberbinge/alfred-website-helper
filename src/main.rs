use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Deserialize)]
#[serde(untagged)]
enum SiteEntry {
    Template {
        prefix: String,
        template: String,
        #[serde(default)]
        icon: Option<String>,
    },
    Static {
        title: String,
        arg: String,
        #[serde(default)]
        icon: Option<String>,
    },
}

#[derive(Serialize)]
struct AlfredItem {
    uid: String,
    title: String,
    subtitle: String,
    arg: String,
    valid: bool,
    icon: AlfredIcon,
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
    let args: Vec<String> = env::args().collect();
    let query = args.get(1).map(|s| s.to_lowercase()).unwrap_or_default();

    let config_path = env::var("HOME").unwrap_or_default() + "/.config/alfred/sites.json";
    let entries: Vec<SiteEntry> = match fs::read_to_string(&config_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(entries) => entries,
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

    let default_icon = format!(
        "{}/.config/alfred/default.png",
        env::var("HOME").unwrap_or_default()
    );

    let mut items: Vec<AlfredItem> = Vec::new();

    for entry in &entries {
        match entry {
            SiteEntry::Template {
                prefix,
                template,
                icon,
            } => {
                // Check if query starts with this prefix
                if let Some(rest) = query.strip_prefix(&prefix.to_lowercase()) {
                    let param = rest.trim();
                    if !param.is_empty() {
                        let url = template.replace("{}", param);
                        items.push(AlfredItem {
                            uid: format!("{}-{}", prefix, param),
                            title: format!("{} {}", prefix, param),
                            subtitle: url.clone(),
                            arg: url,
                            valid: true,
                            icon: AlfredIcon {
                                path: icon.clone().unwrap_or(default_icon.clone()),
                            },
                        });
                    }
                }
            }
            SiteEntry::Static { title, arg, icon } => {
                if query.is_empty() || title.to_lowercase().contains(&query) {
                    items.push(AlfredItem {
                        uid: title.clone(),
                        title: title.clone(),
                        subtitle: arg.clone(),
                        arg: arg.clone(),
                        valid: true,
                        icon: AlfredIcon {
                            path: icon.clone().unwrap_or(default_icon.clone()),
                        },
                    });
                }
            }
        }
    }

    let output = AlfredOutput { items };
    println!(
        "{}",
        serde_json::to_string(&output).unwrap_or_else(|_| {
            r#"{"items": [{"title": "Error", "subtitle": "Failed to serialize JSON", "valid": false}]}"#
                .to_string()
        })
    );
}
