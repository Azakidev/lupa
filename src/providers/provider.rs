/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use crate::window::PikolaunchWindow;

pub trait Provider {
    const PREFIX: char;

    fn prepare(&self, win: &PikolaunchWindow);

    fn hide_entries(&self);

    fn update_entries(&self, query: &str, win: &PikolaunchWindow);
}
