/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use crate::{
    components::{entry::LupaEntry, sidebar::LupaSidebarContent},
    window::LupaWindow,
};

pub trait Provider {
    fn prepare(&self, win: &LupaWindow);

    fn hide_entries(&self);

    fn update_entries(&self, query: &str, win: &LupaWindow);

    fn prefix(&self) -> char;

    fn name(&self) -> &str;
}

pub trait SidebarProvider {
    fn populate_sidebar(&self, entry: &LupaEntry, win: &LupaWindow) -> LupaSidebarContent;
}
