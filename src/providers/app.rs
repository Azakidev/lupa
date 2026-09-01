/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
    env::var,
    eprintln,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use adw::{glib::WeakRef, prelude::*, subclass::prelude::*};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use icon_finder::find_icon;

use crate::{
    components::entry::PikolaunchEntry, providers::provider::Provider, window::PikolaunchWindow,
};

#[derive(Default)]
pub struct AppProvider {
    icon_size: OnceCell<u32>,
    cache: RefCell<HashMap<String, WeakRef<PikolaunchEntry>>>,
    matcher: SkimMatcherV2,
}

impl Provider for AppProvider {
    const PREFIX: char = '#';

    fn prepare(&self, win: &PikolaunchWindow) {
        let mut cache = self.cache.borrow_mut();

        self.icon_size
            .set(win.icon_size())
            .expect("Failed to set icon size");

        let imp = win.imp();
        let results = &imp.results;

        let apps = discover_apps().unwrap_or_default();

        for app in apps {
            let entry = PikolaunchEntry::new_app(app.clone(), win.icon_size());
            results.append(&entry);

            cache.insert(app.name, entry.downgrade());
        }
    }

    fn hide_entries(&self) {
        self.cache
            .borrow()
            .iter()
            .filter_map(|(_, weak)| weak.upgrade())
            .for_each(|entry| entry.set_visible(false));
    }

    fn update_entries(&self, query: &str, win: &PikolaunchWindow) {
        let cache = self.cache.borrow();
        let matcher = &self.matcher;
        let results = win.imp().results.get();

        let query = query.strip_prefix(Self::PREFIX).unwrap_or(query);

        let mut filtered = cache
            .keys()
            .filter(|a| {
                query
                    .to_lowercase()
                    .chars()
                    .map(|c| c.to_string())
                    .all(|c| a.to_lowercase().contains(&c))
            })
            .filter_map(|a| {
                if let Some(score) = matcher.fuzzy_match(&a.to_lowercase(), &query.to_lowercase())
                    && score >= 25
                {
                    Some((a.clone(), score))
                } else {
                    None
                }
            })
            .collect::<Vec<(String, i64)>>();

        filtered.sort_unstable_by_key(|(_, s)| *s);

        let mut prev: Option<WeakRef<PikolaunchEntry>> = None;

        for a in filtered.iter().map(|(a, _)| a.clone()).rev() {
            if let Some(weak) = cache.get(&a)
                && let Some(entry) = weak.upgrade()
            {
                if let Some(prev_weak) = prev {
                    results.reorder_child_after(&entry, prev_weak.upgrade().as_ref());
                }

                entry.set_visible(true);
                prev = Some(entry.downgrade());
            }
        }
    }
}

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

    let Ok(home) = var("HOME") else {
        eprintln!("[Error] Could not find user home, is HOME set?");
        return None;
    };

    let Ok(desktop) = var("XDG_CURRENT_DESKTOP") else {
        eprintln!("[Error] Could not detect desktop, is XDG_CURRENT_DESKTOP set?");
        return None;
    };

    let user_data_path = Path::new(&home).join(".local/share");

    let mut apps: Vec<App> = Vec::new();

    for path in locations
        .split(":")
        .chain([user_data_path.to_str().unwrap()])
    {
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

        if in_main_section && let Some((key, value)) = line.split_once('=') {
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

pub fn find_icon_path(name: &str, size: u32) -> Option<PathBuf> {
    let path = Path::new(name);

    if path.is_absolute() && path.exists() {
        return Some(path.to_path_buf());
    }

    // Android Studio seems to be broken on some themes, prefer the default pixmap icon
    if let Some(path) = find_icon(name, size)
        && name != "android-studio"
    {
        return Some(path);
    }

    let xdg_dirs = xdg::BaseDirectories::new();
    let mut string = String::with_capacity(128);

    for ext in ["svg", "png"] {
        string.clear();
        write!(string, "pixmaps/{}.{}", name, ext).ok()?;
        if let Some(path) = xdg_dirs.find_data_file(&string) {
            return Some(path);
        }
    }

    // If all else fails
    None
}
