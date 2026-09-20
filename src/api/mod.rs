/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

mod clipboard;
mod spawn;

use adw::glib;
use mlua::prelude::*;

use crate::api::{clipboard::clipboard_copy, spawn::spawn};

// Creates a lua context, creates the global lupa table and inserts it into the globals
pub fn prepare_lua(win: &crate::window::LupaWindow) -> LuaResult<mlua::Lua> {
    let lua = mlua::Lua::new();

    let table = lua.create_table()?;

    table.set("spawn", lua.create_function(spawn)?)?;
    table.set(
        "clipboard_copy",
        lua.create_function(glib::clone!(
            #[weak]
            win,
            #[upgrade_or]
            Err(LuaError::RuntimeError("Failed to execute copy".to_string())),
            move |lua, text| clipboard_copy(&win, lua, text)
        ))?,
    )?;

    lua.globals().set("lupa", table)?;

    Ok(lua)
}
