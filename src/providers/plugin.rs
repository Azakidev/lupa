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
use mlua::prelude::*;
use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
};

use crate::{
    components::{entry::LupaEntry, sidebar::LupaSidebarContent},
    providers::provider::{Provider, SidebarProvider},
    window::LupaWindow,
};

#[derive(Default)]
pub struct PluginProvider {
    // Configuration from window
    icon_size: OnceCell<u32>,
    max_entries: OnceCell<u32>,
    // Plugin flags
    support_sidebar: bool,
    sort_results: bool,
    // What makes it work
    cache: RefCell<HashMap<String, WeakRef<LupaEntry>>>,
    matcher: SkimMatcherV2,
    lua: mlua::Lua,
}

impl PluginProvider {
    pub fn new(lua: mlua::Lua) -> Self {
        let support_sidebar = lua.globals().get("SUPPORTS_SIDEBAR").unwrap_or(false);
        let sort_results = lua.globals().get("SORT_RESULTS").unwrap_or(false);

        Self {
            lua,
            support_sidebar,
            sort_results,
            ..Default::default()
        }
    }

    fn get_or_create_entry(
        &self,
        plugin_entry: PluginEntry,
        cache: &mut HashMap<String, WeakRef<LupaEntry>>,
        results: &gtk::Box,
        win: &LupaWindow,
    ) -> LupaEntry {
        let icon_size = self.icon_size.get().copied();

        if let Some(weak) = cache.get(&plugin_entry.name)
            && let Some(entry) = weak.upgrade()
        {
            entry
        } else {
            let name = self.name();
            let icon = plugin_entry
                .icon
                .as_deref()
                .unwrap_or("puzzle-piece-symbolic");

            let provider: Option<Box<dyn SidebarProvider>> = if self.support_sidebar {
                let prov = Self::new(self.lua.clone());
                prov.icon_size
                    .set(self.icon_size.get().copied().unwrap_or(24))
                    .expect("Failed to transfer icon size");
                Some(Box::new(prov))
            } else {
                None
            };

            let entry = LupaEntry::new(
                &plugin_entry.name,
                plugin_entry.description.as_deref(),
                Some(icon),
                false,
                false,
                icon_size,
                provider,
                win,
                glib::clone!(
                    #[weak]
                    win,
                    #[strong(rename_to=lua)]
                    self.lua,
                    #[strong]
                    name,
                    #[strong]
                    plugin_entry,
                    move |_| {
                        win.close();

                        if let Ok(func) = lua.globals().get::<mlua::Function>("EXECUTE_ENTRY")
                            && let Ok(e) = lua.to_value(&plugin_entry)
                            && let Err(e) = func.call::<()>(e)
                        {
                            eprintln!(
                                "[Error] Failed to call EXECUTE_ENTRY on plugin {}: {}",
                                name, e
                            )
                        }
                    }
                ),
            );

            cache.insert(plugin_entry.name.clone(), entry.downgrade());

            results.append(&entry);

            entry
        }
    }
}

impl Provider for PluginProvider {
    fn prefix(&self) -> char {
        self.lua.globals().get("PREFIX").unwrap_or('❓')
    }

    fn name(&self) -> String {
        self.lua.globals().get("NAME").unwrap_or("❓".to_string())
    }

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

        // Don't pass an empty query to lua
        if query.is_empty() {
            return;
        }

        let mut all_entries = Vec::new();

        // Get results from lua land
        if let Ok(func) = self.lua.globals().get::<mlua::Function>("GET_RESULTS") {
            match func.call::<mlua::Table>(query) {
                Ok(result_table)
                    if let Ok(result_entries) = self
                        .lua
                        .from_value::<Vec<PluginEntry>>(result_table.to_value()) =>
                {
                    for plugin_entry in result_entries.iter().take(max_entries).cloned() {
                        let entry = self.get_or_create_entry(
                            plugin_entry.clone(),
                            &mut cache,
                            &results,
                            win,
                        );
                        all_entries.push((plugin_entry, entry));
                    }
                }
                Ok(_) => {
                    eprintln!("[Warn] Got unknown data from plugin {}", self.name());
                    return;
                }
                Err(e) => {
                    eprintln!(
                        "[Warn] Failed to parse result entry from plugin {}: {}",
                        self.name(),
                        e
                    );
                    return;
                }
            }
        } else {
            eprintln!(
                "[Error] Provider {} failed to execute the GET_RESULTS function",
                self.name()
            );
            return;
        }

        if self.sort_results {
            all_entries.sort_unstable_by_key(|(pe, _)| matcher.fuzzy_match(&pe.name, query));
        }

        let mut prev: Option<WeakRef<LupaEntry>> = None;
        for (_, le) in all_entries.iter().rev().take(max_entries) {
            if let Some(prev_weak) = prev {
                results.reorder_child_after(le, prev_weak.upgrade().as_ref());
            }

            le.set_visible(true);
            prev = Some(le.downgrade());
        }
    }
}

impl SidebarProvider for PluginProvider {
    fn populate_sidebar(&self, entry: &LupaEntry, win: &LupaWindow) -> LupaSidebarContent {
        let icon_size = self.icon_size.get().copied().unwrap();
        let name = entry.imp().name.text();
        let comment = entry.imp().comment.text();

        let desc = if !comment.is_empty() {
            Some(comment.as_str())
        } else {
            None
        };

        let sidebar = LupaSidebarContent::new(name.as_str(), desc, None, icon_size, false);

        // Get results from lua land
        if let Ok(func) = self
            .lua
            .globals()
            .get::<mlua::Function>("GET_SIDEBAR_ACTIONS")
        {
            if let Ok(result_table) = func.call::<mlua::Table>(name.to_string())
                && let Ok(result_entries) = self
                    .lua
                    .from_value::<Vec<PluginEntry>>(result_table.to_value())
            {
                for entry in result_entries {
                    let icon = entry.icon.as_deref().unwrap_or("puzzle-piece-symbolic");

                    sidebar.add_action(
                        &entry.name,
                        Some(icon),
                        glib::clone!(
                            #[weak]
                            win,
                            #[strong]
                            name,
                            #[strong]
                            entry,
                            #[strong(rename_to=lua)]
                            self.lua,
                            move |_| {
                                win.close();

                                if let Ok(func) =
                                    lua.globals().get::<mlua::Function>("EXECUTE_SIDEBAR_ACTION")
                                    && let Ok(e) = lua.to_value(&entry)
                                    && let Err(e) = func.call::<()>(e)
                                {
                                    eprintln!(
                                        "[Error] Failed to call EXECUTE_SIDEBAR_ACTION on plugin {}: {}",
                                        name, e
                                    )
                                }
                            }
                        ),
                    );
                }
            }
        } else {
            eprintln!(
                "[Error] Provider {} failed to execute get_results function",
                self.name()
            );
        }

        sidebar
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct PluginEntry {
    name: String,
    description: Option<String>,
    icon: Option<String>,
}
