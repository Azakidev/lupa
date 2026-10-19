/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

mod api;
mod application;
mod components;
mod config;
mod providers;
mod utils;
mod window;

use adw::{gio, glib, prelude::*};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use nix::libc;
use time::macros::format_description;
use tracing::error;
use tracing_subscriber::{FmtSubscriber, fmt::time::LocalTime};

use self::{application::LupaApplication, window::LupaWindow};

static GETTEXT_PACKAGE: &str = "lupa";

pub static DEFAULT_CONFIG: &str = include_str!("../data/default.toml");
pub static EXAMPLE_PLUGIN: &str = include_str!("../data/example_plugin.lua");

fn main() -> glib::ExitCode {
    init_logging();

    // NEVER run Lupa as root
    if unsafe { libc::getuid() } == 0 {
        error!("Lupa should NEVER be run as root");
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

fn init_logging() {
    let level = if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::WARN
    };

    let timer = LocalTime::new(format_description!("[hour]:[minute]:[second]"));

    FmtSubscriber::builder()
        .with_max_level(level)
        .with_timer(timer)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();
}
