/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use std::{
    io,
    os::unix::process::CommandExt,
    process::{Child, Command, Stdio},
};

use gtk::{Widget, prelude::*};

pub fn first_visible_child(container: &gtk::Box) -> Option<Widget> {
    let mut current = container.first_child();

    while let Some(child) = current {
        if child.is_visible() {
            return Some(child);
        }
        current = child.next_sibling();
    }

    None
}

pub fn spawn_with_new_session(command: &mut Command) -> io::Result<Child> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // SAFETY: We are in the "fork-exec gap".
    // We avoid heap allocation and use only async-signal-safe calls.
    unsafe {
        command.pre_exec(|| {
            nix::unistd::setsid()
                .map(|_| ())
                .map_err(|e| io::Error::from_raw_os_error(e as i32))
        });
    }

    command.spawn()
}
