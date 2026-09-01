/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use adw::{gdk::Display, gio, glib, prelude::*, subclass::prelude::*};
use std::cell::OnceCell;

use crate::{PikolaunchWindow, config::PikolaunchConfig};

mod imp {
    use gettextrs::gettext;

    use crate::config::DEFAULT_CONFIG;

    use super::*;

    #[derive(Debug, Default)]
    pub struct PikolaunchApplication {
        pub config: OnceCell<PikolaunchConfig>,
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

            obj.add_main_option(
                "default-config",
                glib::Char::from(b'd'),
                glib::OptionFlags::NONE,
                glib::OptionArg::None,
                &gettext("Print the default configuration"),
                None,
            );
        }
    }

    impl ApplicationImpl for PikolaunchApplication {
        fn activate(&self) {
            let application = self.obj();
            // Get the current window or create one if necessary
            application.load_config_styles();

            let config = application.imp().config.get().unwrap();
            let icon_size = config.aesthetic.entry_size - 4;

            let window = application.active_window().unwrap_or_else(|| {
                let window = PikolaunchWindow::new(&*application, icon_size);
                application.setup_window_config(&window);
                window.upcast()
            });

            window.present();
        }

        fn handle_local_options(
            &self,
            options: &glib::VariantDict,
        ) -> std::ops::ControlFlow<glib::ExitCode> {
            if options.lookup_value("default-config", None).is_some() {
                println!("{}", DEFAULT_CONFIG);
                self.obj().quit();
            }

            std::ops::ControlFlow::Continue(())
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

    fn load_config_styles(&self) {
        let provider = gtk::CssProvider::new();

        let config = self.config();

        let opacity = config.aesthetic.opacity;
        let radius = config.aesthetic.radius;

        let entry_size = config.aesthetic.entry_size;

        provider.load_from_string(&format!(
            ".launcher {{
                background-color: rgb(from var(--window-bg-color) r g b / {opacity});
                border-radius: {radius}px;
            }}

            .launcher_entry {{
                min-height: {entry_size}px;
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

        let is_exact = entries.fract() == 0.;

        // Calculates the spacing of the ScrolledWindow so the following things are true:
        // - Each entry is as big as the user configured
        // - There as as many entries visible as the user configured
        // - If the number of visible is round it'll add a small padding so the last entry has
        //   some breathing room
        let mut size = (entry_size as f32 * entries) + (2.0 * entries);
        if is_exact {
            size += 4.0
        };

        scroller.set_height_request(size as i32);
    }
}
