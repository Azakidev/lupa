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
use emojis::Emoji;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
};

use crate::{components::entry::LupaEntry, providers::provider::Provider, window::LupaWindow};

#[derive(Default)]
pub struct EmojiProvider {
    icon_size: OnceCell<u32>,
    max_entries: OnceCell<u32>,
    cache: RefCell<HashMap<String, WeakRef<LupaEntry>>>,
    matcher: SkimMatcherV2,
}

impl Provider for EmojiProvider {
    const PREFIX: char = '!';

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

        let query = query.strip_prefix(self.prefix()).unwrap_or(query);

        let mut filtered = emojis::iter()
            .filter_map(|e| {
                if let Some(score) = matcher.fuzzy_match(e.shortcode().unwrap_or(e.name()), query)
                    && score >= 60
                {
                    Some((e, score))
                } else {
                    None
                }
            })
            .collect::<Vec<(&Emoji, i64)>>();

        filtered.sort_by_cached_key(|(_, score)| *score);

        let mut prev: Option<WeakRef<LupaEntry>> = None;

        for (emoji, _) in filtered.iter().rev().take(max_entries) {
            let entry = if let Some(weak) = cache.get(emoji.name())
                && let Some(e) = weak.upgrade()
            {
                e
            } else {
                self.create_entry(&mut cache, emoji, win)
            };

            if let Some(prev_weak) = prev {
                results.reorder_child_after(&entry, prev_weak.upgrade().as_ref());
            }

            entry.set_visible(true);
            prev = Some(entry.downgrade());
        }
    }
}

impl EmojiProvider {
    fn create_entry(
        &self,
        cache: &mut HashMap<String, WeakRef<LupaEntry>>,
        emoji: &Emoji,
        win: &LupaWindow,
    ) -> LupaEntry {
        let results = win.imp().results.get();
        let icon_size = self.icon_size.get().copied();

        let emo = emoji.as_str().to_string();

        let comment = if let Some(sc) = emoji.shortcode() {
            &format!("{}, {}", emoji.name(), sc)
        } else {
            emoji.name()
        };

        let entry = LupaEntry::new(
            emoji.as_str(),
            Some(comment),
            Some("smile-symbolic"),
            false,
            false,
            icon_size,
            None,
            win,
            glib::clone!(
                #[weak]
                win,
                #[strong(rename_to=emoji)]
                emo,
                move |b| {
                    b.clipboard().set_text(&emoji);

                    win.close();
                }
            ),
        );

        results.append(&entry);
        cache.insert(emoji.name().to_string(), entry.downgrade());

        entry
    }
}
