/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use std::{collections::HashMap, path::Path, process::Command};

use adw::{
    glib::{self, WeakRef},
    prelude::*,
};

use crate::{
    components::entry::PikolaunchEntry, utils::spawn_with_new_session, window::PikolaunchWindow,
};

pub fn update_file_search_results(
    query: &str,
    cache: &mut HashMap<String, WeakRef<PikolaunchEntry>>,
    win: &PikolaunchWindow,
    results: &gtk::Box,
    icon_size: u32,
) {
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

    present.iter().for_each(|f| {
        let path = Path::new(f);

        if path.exists() && !cache.contains_key(f) {
            generate_file_entry(path, cache, win, results, icon_size);
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

fn generate_file_entry(
    file: &Path,
    cache: &mut HashMap<String, WeakRef<PikolaunchEntry>>,
    win: &PikolaunchWindow,
    results: &gtk::Box,
    icon_size: u32,
) {
    // File exists, we checked, so it should have a name
    let filepath = file.to_str().map(|s| s.to_string()).unwrap();

    let entry = PikolaunchEntry::new_raw(
        file.file_name().and_then(|s| s.to_str()).unwrap(),
        file.to_str(),
        // TODO: Custom icon per file (and maybe thumbnails for images)
        Some("folder-documents-symbolic"),
        Some(icon_size),
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
    results.prepend(&entry);

    cache.insert(filepath, entry.downgrade());
}
