/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use std::{
    env::var,
    eprintln,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

const ICON_SIZES: [i32; 6] = [16, 32, 48, 64, 128, 256];

#[derive(Debug, Default, Clone)]
pub struct App {
    pub name: String,
    pub exec: String,
    pub comment: Option<String>,
    pub icon: Option<String>,
    pub is_flatpak: bool,
}

pub fn discover_apps() -> Option<Vec<App>> {
    let Ok(locations) = var("XDG_DATA_DIRS") else {
        eprintln!("[Error] Could not find application data locations, is XDG_DATA_DIRS set?");
        return None;
    };

    let Ok(desktop) = var("XDG_CURRENT_DESKTOP") else {
        eprintln!("[Error] Could not detect desktop, is XDG_CURRENT_DESKTOP set?");
        return None;
    };

    let mut apps: Vec<App> = Vec::new();

    for path in locations.split(":") {
        let app_dir = Path::new(path).join("applications");

        let is_flatpak = path.contains("flatpak");

        if let Ok(reader) = std::fs::read_dir(&app_dir) {
            let mut found_apps = reader
                .filter_map(|e| -> Option<App> {
                    if let Ok(entry) = e
                        && entry.file_name().to_string_lossy().ends_with("desktop")
                    {
                        let path = entry.path();

                        if let Ok(buf) = fs::read_to_string(path) {
                            parse_desktop_entry(&buf, &desktop, is_flatpak)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<App>>();

            apps.append(&mut found_apps);
        }
    }

    Some(apps)
}

fn parse_desktop_entry(content: &str, current_desktop: &str, is_flatpak: bool) -> Option<App> {
    let mut app = App::default();
    let mut in_main_section = false;

    let mut has_name = false;
    let mut has_exec = false;
    let mut has_type = false;
    let mut should_hide = false;

    app.is_flatpak = is_flatpak;

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') {
            if in_main_section {
                break;
            }
            in_main_section = line == "[Desktop Entry]";
            continue;
        }

        if in_main_section
            && let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "Type" => {
                        if value != "Application" {
                            return None;
                        }
                        has_type = true;
                    }
                    "NoDisplay" | "Hidden" => {
                        if value == "true" {
                            should_hide = true;
                        }
                    }
                    "OnlyShowIn" => {
                        let mut required_desktops = value.split(';').filter(|s| !s.is_empty());
                        let is_match = required_desktops.any(|d| current_desktop == d);

                        if !is_match {
                            should_hide = true;
                        }
                    }
                    "NotShowIn" => {
                        let mut required_desktops = value.split(';').filter(|s| !s.is_empty());
                        let is_match = required_desktops.any(|d| current_desktop == d);

                        if is_match {
                            should_hide = true;
                        }
                    }
                    "Name" => {
                        app.name = value.to_string();
                        has_name = true;
                    }
                    "Exec" => {
                        app.exec = value.to_string();
                        has_exec = true;
                    }
                    "Icon" => app.icon = Some(value.to_string()),
                    "Comment" => app.comment = Some(value.to_string()),
                    _ => {}
                }
            }
    }

    if !should_hide && has_name && has_exec && has_type {
        Some(app)
    } else {
        None
    }
}

pub fn find_icon_path(name: &str) -> Option<PathBuf> {
    let path = Path::new(name);

    if path.is_absolute() && path.exists() {
        return Some(path.to_path_buf());
    }

    let xdg_dirs = xdg::BaseDirectories::new();
    let mut string = String::with_capacity(128);

    write!(string, "icons/hicolor/scalable/apps/{}.svg", name).ok()?;
    if let Some(found_path) = xdg_dirs.find_data_file(&string) {
        return Some(found_path);
    }

    for size in ICON_SIZES {
        string.clear();
        write!(string, "icons/hicolor/{}/apps/{}.png", size, name).ok()?;
        if let Some(found_path) = xdg_dirs.find_data_file(&string) {
            return Some(found_path);
        }
    }

    None
}
