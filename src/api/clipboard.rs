/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use adw::prelude::*;
use mlua::prelude::*;

use crate::{window::LupaWindow};

pub fn clipboard_copy(win: &LupaWindow, _: &mlua::Lua, text: String) -> LuaResult<()> {
    win.clipboard().set_text(&text);
    Ok(())
}
