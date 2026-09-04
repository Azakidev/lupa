/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
    path::Path,
    process::Command,
};

use adw::{
    glib::{self, WeakRef},
    prelude::*,
    subclass::prelude::*,
};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use gettextrs::gettext;
use mime_type::{MimeFormat, MimeType};
use urlencoding::decode;

use crate::{
    components::{entry::LupaEntry, sidebar::LupaSidebarContent},
    providers::provider::{Provider, SidebarProvider},
    utils::spawn_with_new_session,
    window::LupaWindow,
};

#[derive(Default)]
pub struct FileProvider {
    icon_size: OnceCell<u32>,
    max_entries: OnceCell<u32>,
    cache: RefCell<HashMap<String, WeakRef<LupaEntry>>>,
    matcher: SkimMatcherV2,
}

impl Provider for FileProvider {
    const PREFIX: char = '/';

    fn prepare(&self, win: &LupaWindow) {
        self.icon_size
            .set(win.icon_size())
            .expect("Failed to set icon size");
        self.max_entries
            .set(win.max_file_entries())
            .expect("Failed to set file entry limit");
    }

    fn hide_entries(&self) {
        self.cache
            .borrow()
            .iter()
            .filter_map(|(_, weak)| weak.upgrade())
            .for_each(|entry| entry.set_visible(false));
    }

    fn update_entries(&self, query: &str, win: &LupaWindow) {
        let mut cache = self.cache.borrow_mut();
        let max_entries = *self.max_entries.get().unwrap() as usize;
        let matcher = &self.matcher;
        let results = win.imp().results.get();

        // /query/path/with/folder/
        let query = query.strip_prefix(Self::PREFIX).unwrap_or(query);

        if query.is_empty() {
            return;
        }

        let folder_only = query.ends_with(Self::PREFIX);

        // query/path/with/folder/
        let query = query.strip_suffix(Self::PREFIX).unwrap_or(query);
        // query/path/with/folder

        let mut command = Command::new("localsearch");

        command.arg("search").arg(query);

        if folder_only {
            command.arg("--folders");
        }

        let Ok(output) = command.output() else {
            return;
        };

        let Ok(string) = String::from_utf8(output.stdout) else {
            return;
        };

        let mut present = string
            .trim()
            .lines()
            .filter_map(|l| {
                let Ok(res) = decode(l) else {
                    eprintln!("[Warn] Couldn't decode {}", l);
                    return None;
                };
                Some(res.replace("file://", ""))
            })
            .filter_map(|f| {
                let path = Path::new(&f);
                if let Some(name) = path.file_name().and_then(|s| s.to_str())
                    && let Some(score) =
                        matcher.fuzzy_match(&name.to_lowercase(), &query.to_lowercase())
                    && score.is_positive()
                {
                    Some((f, score))
                } else {
                    None
                }
            })
            .collect::<Vec<(String, i64)>>();

        present.sort_by_cached_key(|(_, s)| *s);

        present
            .iter()
            .rev()
            .map(|(f, _)| f)
            .take(max_entries)
            .for_each(|f| {
                let path = Path::new(f);

                if path.exists()
                    && let Some(weak) = cache.get(f)
                    && let Some(entry) = weak.upgrade()
                {
                    entry.set_visible(true);
                } else {
                    self.generate_file_entry(
                        &mut cache,
                        path,
                        win,
                        &results,
                        self.icon_size.get().and_then(|s| Some(*s)),
                    );
                }
            });

        let non_present = cache
            .iter()
            .filter(|(k, _)| {
                !present
                    .iter()
                    .map(|(s, _)| s)
                    .collect::<Vec<&String>>()
                    .contains(k)
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<(String, WeakRef<LupaEntry>)>>();

        non_present.iter().for_each(|(k, weak)| {
            if let Some(entry) = weak.upgrade() {
                results.remove(&entry);
            }
            cache.remove(k);
        });
    }
}

impl SidebarProvider for FileProvider {
    fn populate_sidebar(&self, entry: &LupaEntry, win: &LupaWindow) -> LupaSidebarContent {
        let imp = entry.imp();
        let file = imp.name.text().to_string();
        let path = imp.comment.text().to_string();
        let icon = imp.icon.icon_name().unwrap();
        let size = *self.icon_size.get().unwrap();

        let sidebar = LupaSidebarContent::new(&file, Some(&path), Some(&icon), size, false);

        // Open in browser
        sidebar.add_action(
            &gettext("Open in file browser"),
            Some("external-link-symbolic"),
            glib::clone!(
                #[weak]
                win,
                #[strong(rename_to=filepath)]
                path,
                move |_| {
                    let path = Path::new(&filepath);

                    let mut command = Command::new("xdg-open");

                    if path.is_dir() {
                        command.arg(&filepath);
                    } else {
                        command.arg(&path.parent().unwrap_or(&path));
                    }

                    if let Err(e) = spawn_with_new_session(&mut command) {
                        eprint!("[Error] Failed to open file: {}", e);
                    }

                    win.close();
                }
            ),
        );

        sidebar.add_action(
            &gettext("Copy path"),
            Some("clipboard-symbolic"),
            glib::clone!(
                #[weak]
                win,
                #[strong]
                path,
                move |b| {
                    b.clipboard().set_text(&path);
                    win.close();
                }
            ),
        );

        sidebar
    }
}

impl FileProvider {
    fn generate_file_entry(
        &self,
        cache: &mut HashMap<String, WeakRef<LupaEntry>>,
        file: &Path,
        win: &LupaWindow,
        results: &gtk::Box,
        icon_size: Option<u32>,
    ) -> LupaEntry {
        // File exists, we checked, so it should have a name
        let filepath = file.to_str().map(|s| s.to_string()).unwrap();
        let icon = build_icon(&file);

        let prov = Self::default();
        prov.icon_size
            .set(
                self.icon_size
                    .get()
                    .and_then(|n| Some(*n))
                    .unwrap_or_default(),
            )
            .expect("Failed to copy icon size");

        let entry = LupaEntry::new(
            file.file_name().and_then(|s| s.to_str()).unwrap(),
            file.to_str(),
            Some(icon),
            false,
            false,
            icon_size,
            Some(Box::new(prov)),
            win,
            glib::clone!(
                #[weak]
                win,
                #[strong]
                filepath,
                move |_| {
                    let mut command = Command::new("xdg-open");
                    command.arg(&filepath);

                    if let Err(e) = spawn_with_new_session(&mut command) {
                        eprint!("[Error] Failed to open file: {}", e);
                    }

                    win.close();
                }
            ),
        );

        entry.set_visible(true);
        results.append(&entry);
        cache.insert(filepath, entry.downgrade());

        entry
    }
}

fn build_icon(path: &Path) -> &str {
    let default = "folder-documents-symbolic";

    if path.is_dir() {
        return "document-open-symbolic";
    }

    let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
        return default;
    };

    if ext.contains("md") {
        return "x-office-document-symbolic";
    }

    let Some(mime) = MimeType::from_ext(ext) else {
        return default;
    };

    match mime.to_string() {
        f if f.contains("image") => "image-x-generic-symbolic",
        f if f.contains("audio") => "folder-music-symbolic",
        f if f.contains("video") => "folder-videos-symbolic",
        f if f.contains("text") | f.contains("document") | f.contains("pdf") => {
            "x-office-document-symbolic"
        }
        _ => default,
    }
}
