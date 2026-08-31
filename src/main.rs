/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

mod application;
mod components;
mod providers;
mod config;
mod window;

use self::application::PikolaunchApplication;
use self::window::PikolaunchWindow;

use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::prelude::*;
use gtk::{gio, glib};

static GETTEXT_PACKAGE: &str = "pikolaunch";

fn main() -> glib::ExitCode {
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
    gio::resources_register_include!("pikolaunch.gresource").expect("Could not load resources");

    // Create app
    let app =
        PikolaunchApplication::new("art.fatdawlf.Pikolaunch", &gio::ApplicationFlags::empty());

    app.run()
}
