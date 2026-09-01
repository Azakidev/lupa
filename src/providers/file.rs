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

use crate::{
    components::entry::PikolaunchEntry, providers::provider::Provider,
    utils::spawn_with_new_session, window::PikolaunchWindow,
};

#[derive(Default, Debug)]
pub struct FileProvider {
    icon_size: OnceCell<u32>,
    cache: RefCell<HashMap<String, WeakRef<PikolaunchEntry>>>,
}

impl Provider for FileProvider {
    const PREFIX: char = '/';

    fn prepare(&self, win: &PikolaunchWindow) {
        self.icon_size
            .set(win.icon_size())
            .expect("Failed to set icon size");
    }

    fn hide_entries(&self) {
        self.cache
            .borrow()
            .iter()
            .filter_map(|(_, weak)| weak.upgrade())
            .for_each(|entry| entry.set_visible(false));
    }

    fn update_entries(&self, query: &str, win: &PikolaunchWindow) {
        let mut cache = self.cache.borrow_mut();
        let results = win.imp().results.get();

        let query = query.strip_prefix(Self::PREFIX).unwrap_or(query);

        let Ok(output) = Command::new("localsearch")
            .arg("search")
            .arg(query)
            .output()
        else {
            return;
        };

        let Ok(string) = String::from_utf8(output.stdout) else {
            return;
        };

        let present = string
            .trim()
            .lines()
            .map(|l| l.replace("file://", ""))
            .collect::<Vec<String>>();

        present.iter().rev().for_each(|f| {
            let path = Path::new(f);

            if path.exists() && !cache.contains_key(f) {
                generate_file_entry(
                    path,
                    &mut cache,
                    win,
                    &results,
                    self.icon_size.get().and_then(|s| Some(*s)),
                );
            }
        });

        let non_present = cache
            .iter()
            .filter(|(k, _)| !present.contains(k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<(String, WeakRef<PikolaunchEntry>)>>();

        non_present.iter().for_each(|(k, weak)| {
            if let Some(entry) = weak.upgrade() {
                results.remove(&entry);
            }
            cache.remove(k);
        });
    }
}

fn generate_file_entry(
    file: &Path,
    cache: &mut HashMap<String, WeakRef<PikolaunchEntry>>,
    win: &PikolaunchWindow,
    results: &gtk::Box,
    icon_size: Option<u32>,
) {
    // File exists, we checked, so it should have a name
    let filepath = file.to_str().map(|s| s.to_string()).unwrap();

    let entry = PikolaunchEntry::new_raw(
        file.file_name().and_then(|s| s.to_str()).unwrap(),
        file.to_str(),
        // TODO: Custom icon per file (and maybe thumbnails for images)
        Some("folder-documents-symbolic"),
        icon_size,
        glib::clone!(
            #[weak]
            win,
            #[strong]
            filepath,
            move || {
                // TODO: Open file
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
}
