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
use gtk::glib::{Unichar, char};
use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
};

use crate::{components::entry::LupaEntry, providers::provider::Provider, window::LupaWindow};

const UNICODE_DATA: &str = include_str!("../../data/UnicodeData.txt");

#[derive(Default)]
pub struct CharProvider {
    icon_size: OnceCell<u32>,
    max_entries: OnceCell<u32>,
    characters: OnceCell<Vec<UnicodeIdentifier>>,
    cache: RefCell<HashMap<String, WeakRef<LupaEntry>>>,
    matcher: SkimMatcherV2,
}

impl Provider for CharProvider {
    fn prefix(&self) -> char {
        '!'
    }

    fn name(&self) -> String {
        "Characters".to_string()
    }

    fn prepare(&self, win: &LupaWindow) {
        self.icon_size
            .set(win.icon_size())
            .expect("Failed to set icon size");
        self.max_entries
            .set(win.max_file_entries())
            .expect("Failed to set file entry limit");

        let mut chars = self.discover_chars();

        chars.sort_by_cached_key(|i| i.name.len());
        chars.reverse();

        self.characters
            .set(chars)
            .expect("Failed to set characters");
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
        let max_entries = *self.max_entries.get().unwrap() as usize;
        let matcher = &self.matcher;
        let results = win.imp().results.get();

        let query = query.strip_prefix(self.prefix()).unwrap_or(query);

        let mut filtered = self
            .characters
            .get()
            .unwrap()
            .iter()
            .filter_map(|i| {
                let name_score = matcher
                    .fuzzy_match(&i.name.to_lowercase(), &query.to_lowercase())
                    .unwrap_or(0);

                let alias_score = matcher
                    .fuzzy_match(&i.alias.to_lowercase(), &query.to_lowercase())
                    .unwrap_or(0);

                if name_score >= 60 || alias_score >= 60 {
                    Some((i, name_score.max(alias_score)))
                } else {
                    None
                }
            })
            .collect::<Vec<(&UnicodeIdentifier, i64)>>();

        filtered.sort_by_cached_key(|(_, score)| *score);

        let mut prev: Option<WeakRef<LupaEntry>> = None;

        for (identifier, _) in filtered.iter().rev().take(max_entries) {
            let entry = if let Some(weak) = cache.get(&identifier.character.to_string())
                && let Some(e) = weak.upgrade()
            {
                e
            } else {
                self.create_entry(&mut cache, identifier, win)
            };

            if let Some(prev_weak) = prev {
                results.reorder_child_after(&entry, prev_weak.upgrade().as_ref());
            }

            entry.set_visible(true);
            prev = Some(entry.downgrade());
        }
    }
}

impl CharProvider {
    fn discover_chars(&self) -> Vec<UnicodeIdentifier> {
        UNICODE_DATA
            .lines()
            .filter_map(UnicodeIdentifier::from_unicode_data_line)
            .collect()
    }

    fn create_entry(
        &self,
        cache: &mut HashMap<String, WeakRef<LupaEntry>>,
        identifier: &UnicodeIdentifier,
        win: &LupaWindow,
    ) -> LupaEntry {
        let results = win.imp().results.get();
        let icon_size = self.icon_size.get().copied();

        let char = identifier.character.to_string();

        let comment = if !identifier.alias.is_empty() {
            format!("{}; {}", identifier.name, identifier.alias)
        } else {
            identifier.name.clone()
        };

        let words: Vec<_> = comment.split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();

        let entry = LupaEntry::new(
            &char,
            Some(&words.join(" ")),
            Some("language-symbolic"),
            false,
            false,
            icon_size,
            None,
            win,
            glib::clone!(
                #[weak]
                win,
                #[strong]
                char,
                move |b| {
                    b.clipboard().set_text(&char);

                    win.close();
                }
            ),
        );

        results.append(&entry);
        cache.insert(char.to_string(), entry.downgrade());

        entry
    }
}

#[derive(Clone, Debug)]
struct UnicodeIdentifier {
    character: char,
    name: String,
    alias: String,
}

impl UnicodeIdentifier {
    fn from_unicode_data_line(line: &str) -> Option<Self> {
        let data: Vec<_> = line.split(';').collect();

        let char = data.first().copied().unwrap();
        let name = data.get(1).copied().unwrap_or("");
        let alias = data.get(12).copied().unwrap_or("");

        if let Ok(hex) = u32::from_str_radix(char, 16)
            && let Some(c) = char::from_u32(hex)
            && !name.contains("Private Use")
            && !(c.is_control() || c.is_zero_width())
            && c.is_defined()
            && emojis::get(&c.to_string()).is_none()
        {
            return Some(Self {
                character: c,
                name: name.to_string(),
                alias: alias.to_string(),
            });
        }

        None
    }
}
