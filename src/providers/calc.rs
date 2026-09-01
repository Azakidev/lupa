/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use adw::{glib, prelude::*};
use evalexpr::*;

use crate::{components::entry::PikolaunchEntry, window::PikolaunchWindow};

static OPERANDS: &[&str] = &["+", "-", "*", "/", "^", "%", "<", ">", "=", "&", "|"];

// This function should evaluate the expression and somehow update and show a math entry in the launcher
// when applied called
pub fn update_calc_results(
    query: &str,
    icon_size: u32,
    win: &PikolaunchWindow,
) -> Option<PikolaunchEntry> {
    if query.len() < 2 {
        return None;
    };

    // Return if it's a constant
    if !OPERANDS.iter().any(|op| query.contains(op)) {
        return None;
    }

    let float_adjusted = if query.contains("/") && !query.contains(".") {
        format!("{query}.")
    } else {
        query.to_string()
    };

    if let Ok(result) = eval(&float_adjusted)
        && !result.is_empty()
    {
        return Some(generate_calc_entry(query, result, icon_size, win));
    };

    None
}

fn generate_calc_entry(
    query: &str,
    val: Value,
    icon_size: u32,
    win: &PikolaunchWindow,
) -> PikolaunchEntry {
    let result = val.to_string();
    let clipboard = win.clipboard();

    let entry = PikolaunchEntry::new_raw(
        &result,
        Some(query),
        Some("accessories-calculator-symbolic"),
        Some(icon_size),
        glib::clone!(
            #[weak]
            win,
            #[strong]
            result,
            move || {
                clipboard.set_text(&result);
                win.close();
            }
        ),
    );

    entry.set_visible(true);

    entry
}
