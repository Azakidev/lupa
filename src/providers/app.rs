/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use adw::{
    glib::{self, WeakRef},
    prelude::*,
    subclass::prelude::*,
};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use gettextrs::gettext;
use gtk::glib::{KeyFile, KeyFileFlags};
use icon_finder::find_icon;
use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
    env::var,
    eprintln,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    components::{entry::LupaEntry, sidebar::LupaSidebarContent},
    providers::provider::{Provider, SidebarProvider},
    utils::spawn_with_new_session,
    window::LupaWindow,
};

#[derive(Default)]
pub struct AppProvider {
    icon_size: OnceCell<u32>,
    max_entries: OnceCell<u32>,
    apps: OnceCell<Vec<App>>,
    cache: RefCell<HashMap<String, WeakRef<LupaEntry>>>,
    matcher: SkimMatcherV2,
}

impl Provider for AppProvider {
    fn prefix(&self) -> char {
        '#'
    }

    fn name(&self) -> &str {
        "App"
    }

    fn prepare(&self, win: &LupaWindow) {
        self.icon_size
            .set(win.icon_size())
            .expect("Failed to set icon size");
        self.max_entries
            .set(win.max_file_entries())
            .expect("Failed to set file entry limit");

        let apps = discover_apps().unwrap_or_default();
        self.apps.set(apps).expect("Failed to set apps");
    }

    fn hide_entries(&self) {
        self.cache
            .borrow()
            .values()
            .filter_map(|weak| weak.upgrade())
            .for_each(|entry| entry.set_visible(false));
    }

    fn update_entries(&self, query: &str, win: &LupaWindow) {
        let mut cache = self.cache.borrow_mut();
        let matcher = &self.matcher;
        let results = win.imp().results.get();
        let max_entries = *self.max_entries.get().unwrap() as usize;

        let Some(apps) = self.apps.get() else {
            return;
        };

        let query = query.strip_prefix(self.prefix()).unwrap_or(query);

        let mut filtered = apps
            .iter()
            .map(|app| (app.name.clone(), app.tryexec.clone(), app.keywords.clone()))
            .filter(|(name, tryexec, kw)| {
                query
                    .to_lowercase()
                    .chars()
                    .map(|c| c.to_string())
                    .all(|c| {
                        name.to_lowercase().contains(&c)
                            || tryexec.to_lowercase().contains(&c)
                            || kw.to_lowercase().contains(&c)
                    })
            })
            .filter_map(|(name, tryexec, kw)| {
                let name_score = matcher
                    .fuzzy_match(&name.to_lowercase(), &query.to_lowercase())
                    .unwrap_or(0);

                let try_exec_score = matcher
                    .fuzzy_match(&tryexec.to_lowercase(), &query.to_lowercase())
                    .unwrap_or(0);

                let kw_score = kw
                    .split(";")
                    .filter(|k| !k.is_empty())
                    .filter_map(|k| matcher.fuzzy_match(&k.to_lowercase(), &query.to_lowercase()))
                    .max()
                    .unwrap_or(0);

                if name_score >= 25 || try_exec_score >= 25 || kw_score >= 25 {
                    Some((name.clone(), name_score.max(kw_score).max(try_exec_score)))
                } else {
                    None
                }
            })
            .collect::<Vec<(String, i64)>>();

        filtered.sort_unstable_by_key(|(_, s)| *s);

        let mut prev: Option<WeakRef<LupaEntry>> = None;

        for a in filtered
            .iter()
            .map(|(a, _)| a.clone())
            .take(max_entries)
            .rev()
        {
            let entry = if let Some(weak) = cache.get(&a) {
                weak.upgrade()
            } else {
                if let Some(app) = apps.iter().find(|app| app.name == a) {
                    let entry = self.make_entry(&mut cache, app.clone(), win);
                    results.append(&entry);
                    Some(entry)
                } else {
                    None
                }
            };

            if let Some(entry) = entry {
                if let Some(prev_weak) = prev {
                    results.reorder_child_after(&entry, prev_weak.upgrade().as_ref());
                }

                entry.set_visible(true);
                prev = Some(entry.downgrade());
            }
        }
    }
}

impl SidebarProvider for AppProvider {
    fn populate_sidebar(&self, entry: &LupaEntry, win: &LupaWindow) -> LupaSidebarContent {
        let apps = self.apps.get().unwrap(); // Cannot be empty, already selected an app
        let app = apps
            .iter()
            .find(|a| a.name == entry.imp().name.text().as_str())
            .unwrap();

        let comment = app.comment.as_deref();

        let icon_name = app.icon.clone().unwrap_or("".to_string());
        let icon_size = self.icon_size.get().copied().unwrap();

        let icon = find_icon_path(&icon_name, icon_size);

        let icon = if let Some(icon_path) = icon {
            icon_path.to_string_lossy().to_string()
        } else {
            "".to_owned()
        };

        let sidebar = LupaSidebarContent::new(&app.name, comment, Some(&icon), icon_size, true);

        for action in &app.actions {
            let icon_name = action
                .icon
                .clone()
                .unwrap_or("external-link-symbolic".to_string());

            sidebar.add_action(
                &action.name,
                Some(&icon_name),
                glib::clone!(
                    #[weak]
                    win,
                    #[strong]
                    action,
                    move |_| {
                        let raw_command: Vec<_> = action
                            .exec
                            .split_whitespace()
                            .filter(|chunk| !chunk.is_empty() && !chunk.starts_with("%"))
                            .collect();

                        let [binary, args @ ..] = raw_command.as_slice() else {
                            return;
                        };

                        let mut command = Command::new(binary);
                        command.args(args);

                        if let Err(e) = spawn_with_new_session(&mut command) {
                            eprintln!("Failed to spawn process: {}", e);
                            return;
                        }
                        win.close();
                    }
                ),
            );
        }

        // Open .desktop location
        sidebar.add_action(
            &gettext("Open entry location"),
            Some("document-open-symbolic"),
            glib::clone!(
                #[weak]
                win,
                #[strong(rename_to=filepath)]
                app.location,
                move |_| {
                    let path = Path::new(&filepath);

                    let mut command = Command::new("xdg-open");

                    if path.is_dir() {
                        command.arg(&filepath);
                    } else {
                        command.arg(path.parent().unwrap_or(path));
                    }

                    if let Err(e) = spawn_with_new_session(&mut command) {
                        eprint!("[Error] Failed to open file: {}", e);
                    }

                    win.close();
                }
            ),
        );

        sidebar
    }
}

impl AppProvider {
    fn make_entry(
        &self,
        cache: &mut HashMap<String, WeakRef<LupaEntry>>,
        app: App,
        win: &LupaWindow,
    ) -> LupaEntry {
        let comment = app.comment.as_deref();

        let provider = Self::default();
        provider
            .icon_size
            .set(self.icon_size.get().copied().unwrap_or(24))
            .expect("Failed to transfer icon size");

        if let Some(apps) = self.apps.get() {
            provider
                .apps
                .set(apps.clone())
                .expect("Failed to transfer apps");
        }

        let entry = LupaEntry::new(
            &app.name,
            comment,
            Some(""),
            true,
            app.is_flatpak,
            self.icon_size.get().copied(),
            Some(Box::new(provider)),
            win,
            glib::clone!(
                #[weak]
                win,
                #[strong]
                app,
                move |_| {
                    let raw_command: Vec<_> = app
                        .exec
                        .split_whitespace()
                        .filter(|chunk| !chunk.is_empty() && !chunk.starts_with("%"))
                        .collect();

                    let [binary, args @ ..] = raw_command.as_slice() else {
                        return;
                    };

                    let mut command = Command::new(binary);
                    command.args(args);

                    if let Err(e) = spawn_with_new_session(&mut command) {
                        eprintln!("Failed to spawn process: {}", e);
                        return;
                    }

                    win.close();
                }
            ),
        );

        entry
            .imp()
            .comment
            .set_ellipsize(gtk::pango::EllipsizeMode::End);

        glib::spawn_future_local(glib::clone!(
            #[weak]
            entry,
            #[strong(rename_to=icon_name)]
            app.icon.unwrap_or("".to_string()),
            #[strong(rename_to=icon_size)]
            self.icon_size.get().copied().unwrap(),
            async move {
                let icon = find_icon_path(&icon_name, icon_size);

                let icon = if let Some(icon_path) = icon {
                    icon_path.to_string_lossy().to_string()
                } else {
                    "".to_owned()
                };

                entry.imp().icon.set_from_file(Some(icon));
            }
        ));

        cache.insert(app.name, entry.downgrade());
        entry
    }
}

#[derive(Debug, Default, Clone)]
pub struct App {
    pub location: String,
    pub name: String,
    pub exec: String,
    pub tryexec: String,
    pub comment: Option<String>,
    pub keywords: String,
    pub icon: Option<String>,
    pub is_flatpak: bool,
    pub actions: Vec<AppAction>,
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct AppAction {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
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
        .split(':')
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

                        if let Ok(buf) = fs::read_to_string(&path) {
                            parse_desktop_entry(&buf, &path.to_string_lossy(), &desktop, is_flatpak)
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

fn parse_desktop_entry(
    content: &str,
    location: &str,
    current_desktop: &str,
    is_flatpak: bool,
) -> Option<App> {
    let mut app = App::default();
    let mut action = AppAction::default();

    let mut in_main_section = false;
    let mut in_action_section = false;

    let mut has_name = false;
    let mut has_exec = false;
    let mut has_type = false;
    let mut should_hide = false;

    let mut section_name = String::new();

    let key_file = KeyFile::new();

    if let Err(e) = key_file.load_from_file(location, KeyFileFlags::NONE) {
        eprintln!(
            "Couldn't parse desktop file for {} as keyfile: {}",
            location, e
        );
        return None;
    };

    app.is_flatpak = is_flatpak;
    app.location = location.to_string();

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') {
            section_name = line.replace(['[', ']'], "");

            if in_action_section {
                if action.name != String::default() && action.exec != String::default() {
                    app.actions.push(action.clone());
                }
                action = AppAction::default();
            }

            in_main_section = line == "[Desktop Entry]";
            in_action_section = line.starts_with("[Desktop Action");

            if in_action_section {
                action.id = line
                    .replace("[Desktop Action", "")
                    .replace("]", "")
                    .to_string();
            }
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
                    let localised = key_file
                        .locale_string("Desktop Entry", "Name", None)
                        .map(|s| s.to_string())
                        .unwrap_or(value.to_string());
                    app.name = localised;
                    has_name = true;
                }
                "Exec" => {
                    app.exec = value.to_string();
                    has_exec = true;
                }
                "TryExec" => {
                    app.tryexec = value.to_string();
                }
                "Icon" => app.icon = Some(value.to_string()),
                "Comment" => {
                    let localised = key_file
                        .locale_string(&section_name, "Comment", None)
                        .map(|s| s.to_string())
                        .unwrap_or(value.to_string());
                    app.comment = Some(localised)
                }
                "Keywords" => {
                    let localised = key_file
                        .locale_string(&section_name, "Keywords", None)
                        .map(|s| s.to_string())
                        .unwrap_or(value.to_string());
                    app.keywords = localised;
                }
                _ => {} // No-op
            }
        }

        if in_action_section && let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "Name" => {
                    action.name = key_file
                        .locale_string(&section_name, "Name", None)
                        .map(|s| s.to_string())
                        .unwrap_or(value.to_string())
                }
                "Icon" => action.icon = Some(value.to_string()),
                "Exec" => action.exec = value.to_string(),
                _ => {} // No-op
            }
        }
    }

    if !should_hide && has_name && has_exec && has_type {
        if action != AppAction::default() {
            app.actions.push(action);
        }

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

    // Android Studio seems to be broken on some themes, prefer the default icon
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
