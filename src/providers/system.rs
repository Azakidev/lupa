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
use std::cell::{OnceCell, RefCell};

use crate::{components::entry::LupaEntry, providers::provider::Provider, window::LupaWindow};

#[derive(Default)]
pub struct SystemProvider {
    icon_size: OnceCell<u32>,
    cache: RefCell<Vec<WeakRef<LupaEntry>>>,
    matcher: SkimMatcherV2,
}

impl Provider for SystemProvider {
    const PREFIX: char = '*';

    fn prepare(&self, win: &LupaWindow) {
        self.icon_size
            .set(win.icon_size())
            .expect("Failed to set icon size");

        self.setup_entries(win);
    }

    fn hide_entries(&self) {
        self.cache
            .borrow()
            .iter()
            .filter_map(|weak| weak.upgrade())
            .for_each(|entry| entry.set_visible(false));
    }

    fn update_entries(&self, query: &str, win: &LupaWindow) {
        let cache = self.cache.borrow_mut();
        let matcher = &self.matcher;
        let results = win.imp().results.get();

        let is_explicit = query.starts_with(self.prefix());
        let query = query.strip_prefix(self.prefix()).unwrap_or(query);

        let mut filtered = cache
            .iter()
            .filter_map(|weak| {
                if let Some(e) = weak.upgrade()
                    && let Some(score) = matcher.fuzzy_match(&e.imp().name.text(), query)
                    && if is_explicit {
                        score.is_positive()
                    } else {
                        score >= 65
                    }
                {
                    Some((e, score))
                } else {
                    None
                }
            })
            .collect::<Vec<(LupaEntry, i64)>>();

        filtered.sort_by_cached_key(|(_, score)| *score);

        let mut prev: Option<WeakRef<LupaEntry>> = None;

        for (entry, _) in filtered {
            if let Some(prev_weak) = prev {
                results.reorder_child_after(&entry, prev_weak.upgrade().as_ref());
            }

            entry.set_visible(true);
            prev = Some(entry.downgrade());
        }
    }
}

impl SystemProvider {
    fn setup_entries(&self, win: &LupaWindow) {
        let mut cache = self.cache.borrow_mut();
        let results = &win.imp().results.get();
        let icon_size = self.icon_size.get().copied();

        setup_shutdown_entry(win, &mut cache, results, icon_size);
        setup_reboot_entry(win, &mut cache, results, icon_size);
        setup_sleep_entry(win, &mut cache, results, icon_size);
        setup_hibernate_entry(win, &mut cache, results, icon_size);
    }
}

fn setup_shutdown_entry(
    win: &LupaWindow,
    cache: &mut Vec<WeakRef<LupaEntry>>,
    results: &gtk::Box,
    icon_size: Option<u32>,
) {
    let entry = LupaEntry::new(
        &gettext("Shutdown"),
        None,
        Some("system-shutdown-symbolic"),
        false,
        false,
        icon_size,
        None,
        win,
        glib::clone!(
            #[weak]
            win,
            move |_| {
                if let Err(e) = system_shutdown::shutdown() {
                    eprintln!("[Error] Failed to shutdown: {}", e);
                };

                win.close();
            }
        ),
    );

    results.prepend(&entry);
    cache.push(entry.downgrade());
}

fn setup_reboot_entry(
    win: &LupaWindow,
    cache: &mut Vec<WeakRef<LupaEntry>>,
    results: &gtk::Box,
    icon_size: Option<u32>,
) {
    let entry = LupaEntry::new(
        &gettext("Reboot"),
        None,
        Some("system-reboot-symbolic"),
        false,
        false,
        icon_size,
        None,
        win,
        glib::clone!(
            #[weak]
            win,
            move |_| {
                if let Err(e) = system_shutdown::reboot() {
                    eprintln!("[Error] Failed to reboot: {}", e);
                };

                win.close();
            }
        ),
    );

    results.prepend(&entry);
    cache.push(entry.downgrade());
}

fn setup_hibernate_entry(
    win: &LupaWindow,
    cache: &mut Vec<WeakRef<LupaEntry>>,
    results: &gtk::Box,
    icon_size: Option<u32>,
) {
    let entry = LupaEntry::new(
        &gettext("Hibernate"),
        None,
        Some("background-app-sleepyface-symbolic"),
        false,
        false,
        icon_size,
        None,
        win,
        glib::clone!(
            #[weak]
            win,
            move |_| {
                if let Err(e) = system_shutdown::hibernate() {
                    eprintln!("[Error] Failed to hibernate: {}", e);
                };

                win.close();
            }
        ),
    );

    results.prepend(&entry);
    cache.push(entry.downgrade());
}

fn setup_sleep_entry(
    win: &LupaWindow,
    cache: &mut Vec<WeakRef<LupaEntry>>,
    results: &gtk::Box,
    icon_size: Option<u32>,
) {
    let entry = LupaEntry::new(
        &gettext("Sleep"),
        None,
        Some("background-app-sleepyface-symbolic"),
        false,
        false,
        icon_size,
        None,
        win,
        glib::clone!(
            #[weak]
            win,
            move |_| {
                if let Err(e) = system_shutdown::sleep() {
                    eprintln!("[Error] Failed to sleep: {}", e);
                };

                win.close();
            }
        ),
    );

    results.prepend(&entry);
    cache.push(entry.downgrade());
}
