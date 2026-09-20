/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use mlua::prelude::*;
use std::process::Command;

use crate::utils::spawn_with_new_session;

pub fn spawn(_: &mlua::Lua, exec: String) -> LuaResult<()> {
    let raw_command: Vec<_> = exec
        .lines()
        .filter(|chunk| !chunk.is_empty() && !chunk.starts_with("%"))
        .collect();

    let [binary, args @ ..] = raw_command.as_slice() else {
        return Err(LuaError::RuntimeError(
            "Failed to parse command".to_string(),
        ));
    };

    let mut command = Command::new(binary);
    command.args(args);

    if let Err(e) = spawn_with_new_session(&mut command) {
        eprintln!("Failed to spawn process {}: {}", binary, e);
    }
    Ok(())
}
