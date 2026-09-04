/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::{OnceCell, RefCell};

use adw::{
    glib::{self, WeakRef},
    prelude::*,
    subclass::prelude::*,
};
use evalexpr::*;

use crate::{components::entry::LupaEntry, providers::provider::Provider, window::LupaWindow};

static OPERANDS: &[&str] = &["+", "-", "*", "/", "^", "%", "<", ">", "=", "&", "|"];

#[derive(Default, Debug)]
pub struct CalcProvider {
    icon_size: OnceCell<u32>,
    cache: RefCell<Option<WeakRef<LupaEntry>>>,
}

impl Provider for CalcProvider {
    const PREFIX: char = '=';

    fn prepare(&self, win: &LupaWindow) {
        self.icon_size
            .set(win.icon_size())
            .expect("Failed to set icon size");
    }

    fn hide_entries(&self) {
        let cache = self.cache.borrow();
        if let Some(weak) = cache.as_ref()
            && let Some(entry) = weak.upgrade()
        {
            entry.set_visible(false);
        };
    }

    fn update_entries(&self, query: &str, win: &LupaWindow) {
        let query = query.strip_prefix(Self::PREFIX).unwrap_or(query);

        if query.len() < 2 {
            return;
        };

        // Return if it's a constant
        if !OPERANDS.iter().any(|op| query.contains(op)) {
            return;
        }

        let float_adjusted = if query.contains("/") && !query.contains(".") {
            format!("{query}.")
        } else {
            query.to_string()
        };

        if let Ok(result) = eval(&float_adjusted)
            && !result.is_empty()
        {
            let entry = generate_calc_entry(query, result, *self.icon_size.get().unwrap(), win);
            self.swap_entry(win, &entry);
        };
    }
}

impl CalcProvider {
    fn swap_entry(&self, win: &LupaWindow, entry: &LupaEntry) {
        let results = win.imp().results.get();

        if let Some(weak) = self.cache.borrow().as_ref()
            && let Some(old) = weak.upgrade()
        {
            results.remove(&old);
        }

        results.prepend(entry);
        let weak = entry.downgrade();
        self.cache.replace(Some(weak));
    }
}

fn generate_calc_entry(query: &str, val: Value, icon_size: u32, win: &LupaWindow) -> LupaEntry {
    let result = val.to_string();

    let entry = LupaEntry::new(
        &result,
        Some(query),
        Some("equals-symbolic"),
        false,
        false,
        Some(icon_size),
        None,
        win,
        glib::clone!(
            #[weak]
            win,
            #[strong]
            result,
            move |_| {
                win.clipboard().set_text(&result);
                win.close();
            }
        ),
    );

    entry.imp().icon.add_css_class("dimmed");
    entry.set_visible(true);

    entry
}
