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

static MATH_FUNCS: &[&str] = &[
    "ln", "log", "log2", "log10", "exp", "exp2", "pow", "cos", "acos", "cosh", "acosh", "sin",
    "asin", "sinh", "asinh", "tan", "atan", "atan2", "tanh", "atanh", "sqrt", "cbrt", "hypot",
    "abs",
];

#[derive(Default, Debug)]
pub struct CalcProvider {
    icon_size: OnceCell<u32>,
    cache: RefCell<Option<WeakRef<LupaEntry>>>,
}

impl Provider for CalcProvider {
    fn prefix(&self) -> char {
        '='
    }

    fn name(&self) -> String {
        "Calc".to_string()
    }

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
        let query = query.strip_prefix(self.prefix()).unwrap_or(query);

        // You can't make a math operation that's 2 characters long
        // So skip the calculation
        if query.len() < 3 {
            return;
        };

        let mut query = query.to_string();

        // Automatically inject the math:: prefix for builtin functions
        MATH_FUNCS.iter().for_each(|func| {
            query = query.replace(func, &format!("math::{func}"));
        });

        // Avoid duplicate prefixes
        query = query.replace("math::math::", "math::");

        let context = HashMapContext::<DefaultNumericTypes>::new();

        let float_adjusted = if query.contains("/") && !query.contains(".") {
            format!("{query}.")
        } else {
            query.to_string()
        };

        if let Ok(result) = eval_with_context(&float_adjusted, &context)
            // Check it's not an empty result (invalid operation) or a constant
            && !result.is_empty()
            && result.to_string() != query
        {
            let entry = generate_calc_entry(&query, result, *self.icon_size.get().unwrap(), win);
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
