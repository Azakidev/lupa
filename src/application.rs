/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use adw::{gio, glib, prelude::*, subclass::prelude::*};
use gtk::gdk::Display;

use crate::{
    PikolaunchWindow,
    config::PikolaunchConfig,
    providers::app::{App, discover_apps},
};

mod imp {

    use std::cell::OnceCell;

    use crate::{config::PikolaunchConfig, providers::app::App};

    use super::*;

    #[derive(Debug, Default)]
    pub struct PikolaunchApplication {
        pub config: OnceCell<PikolaunchConfig>,

        // Providers
        pub apps: OnceCell<Vec<App>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PikolaunchApplication {
        const NAME: &'static str = "PikolaunchApplication";
        type Type = super::PikolaunchApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for PikolaunchApplication {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["Escape"]);

            obj.load_providers();
        }
    }

    impl ApplicationImpl for PikolaunchApplication {
        fn activate(&self) {
            let application = self.obj();
            // Get the current window or create one if necessary
            application.load_config_styles();

            let window = application.active_window().unwrap_or_else(|| {
                let window = PikolaunchWindow::new(&*application);
                application.setup_window_config(&window);
                window.upcast()
            });

            window.present();
        }
    }

    impl GtkApplicationImpl for PikolaunchApplication {}
    impl AdwApplicationImpl for PikolaunchApplication {}
}

glib::wrapper! {
    pub struct PikolaunchApplication(ObjectSubclass<imp::PikolaunchApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PikolaunchApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        let app: PikolaunchApplication = glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .property("resource-base-path", "/art/fatdawlf/Pikolaunch")
            .build();

        let config = PikolaunchConfig::load_config();
        app.imp().config.set(config).expect("Could not set config");

        app
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();

        self.add_action_entries([quit_action]);
    }

    pub fn config(&self) -> &PikolaunchConfig {
        self.imp().config.get().unwrap()
    }

    pub fn apps(&self) -> &Vec<App> {
        self.imp().apps.get().unwrap()
    }

    fn load_config_styles(&self) {
        let provider = gtk::CssProvider::new();

        let config = self.config();

        let opacity = config.aesthetic.opacity;
        let radius = config.aesthetic.radius;

        let entry_size = config.aesthetic.entry_size;
        let img_size = entry_size - 4;

        provider.load_from_string(&format!(
            ".launcher {{
                background-color: rgb(from var(--window-bg-color) r g b / {opacity});
                border-radius: {radius}px;
            }}

            .launcher_entry {{
                min-height: {entry_size}px;
            }}

            .entry_image {{
                min-height: {img_size}px;
                min-width:  {img_size}px;
            }}
            "
        ));

        gtk::style_context_add_provider_for_display(
            &Display::default().unwrap(),
            &provider.clone(),
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    fn setup_window_config(&self, win: &PikolaunchWindow) {
        let imp = win.imp();
        let scroller = imp.scroller.get();

        let config = self.config();

        let entry_size = config.aesthetic.entry_size;
        let entries = config.aesthetic.entries;
        scroller.set_height_request((entry_size * entries) as i32);
    }

    fn load_providers(&self) {
        let imp = self.imp();

        let apps = discover_apps().unwrap_or_default();
        imp.apps.set(apps).expect("Failed to set apps provider");
    }
}
