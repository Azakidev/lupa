/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

mod application;
mod components;
mod config;
mod providers;
mod utils;
mod window;

use self::application::LupaApplication;
use self::window::LupaWindow;

use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::prelude::*;
use gtk::{gio, glib};
use nix::libc;

static GETTEXT_PACKAGE: &str = "lupa";

pub static DEFAULT_CONFIG: &str = include_str!("../data/default.toml");
pub static EXAMPLE_PLUGIN: &str = include_str!("../data/example_plugin.lua");

fn main() -> glib::ExitCode {
    // NEVER run Lupa as root
    if unsafe { libc::getuid() } == 0 {
        eprintln!("[Error] Lupa should NEVER be run as root");
        return glib::ExitCode::new(255);
    }

    // Set up gettext translations
    let locale_dir = if cfg!(debug_assertions) {
        format!("{}/locale", env!("OUT_DIR"))
    } else {
        "/usr/share/locale".to_string()
    };

    bindtextdomain(GETTEXT_PACKAGE, locale_dir).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    // Load resources
    gio::resources_register_include!("lupa.gresource").expect("Could not load resources");

    // Create app
    let app = LupaApplication::new("art.fatdawlf.Lupa", &gio::ApplicationFlags::empty());

    app.run()
}
