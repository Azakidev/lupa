/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use adw::{
    gdk::Display,
    gio,
    glib::{self, VariantTy},
    prelude::*,
    subclass::prelude::*,
};
use gettextrs::gettext;
use std::{cell::OnceCell, path::Path};

use crate::{DEFAULT_CONFIG, EXAMPLE_PLUGIN, LupaWindow, config::LupaConfig};

mod imp {

    use super::*;

    #[derive(Debug, Default)]
    pub struct LupaApplication {
        pub config_path_override: OnceCell<String>,
        pub config: OnceCell<LupaConfig>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LupaApplication {
        const NAME: &'static str = "LupaApplication";
        type Type = super::LupaApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for LupaApplication {
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

            obj.add_main_option(
                "init-plugin",
                glib::Char::from(b'p'),
                glib::OptionFlags::NONE,
                glib::OptionArg::None,
                &gettext("Print an example plugin starter"),
                None,
            );

            obj.add_main_option(
                "config-path",
                glib::Char::from(b'c'),
                glib::OptionFlags::NONE,
                glib::OptionArg::String,
                &gettext("Override configuration path"),
                None,
            );
        }
    }

    impl ApplicationImpl for LupaApplication {
        fn activate(&self) {
            let application = self.obj();
            application.load_config_styles();

            let config = application.imp().config.get().unwrap();

            // Get the current window or create one if necessary
            let window = application.active_window().unwrap_or_else(|| {
                let window = LupaWindow::new(&*application, config);
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

            if options.lookup_value("init-plugin", None).is_some() {
                println!("{}", EXAMPLE_PLUGIN);
                self.obj().quit();
            }

            if let Some(var) = options.lookup_value("config-path", Some(VariantTy::STRING)) {
                if let Some(val) = var.str() {
                    let path = Path::new(val);

                    if path.exists()
                        && path.is_dir()
                        && let Ok(mut dir) = std::fs::read_dir(path)
                        && dir.any(|f| {
                            f.is_ok_and(|entry| {
                                entry.file_name().to_string_lossy().contains("conf.toml")
                            })
                        })
                    {
                        self.config_path_override
                            .set(val.to_string())
                            .expect("[Error] Failed to override config path");
                    } else {
                        eprintln!(
                            "[Warning] Selected config path doesn't exist or is not a directory, using default"
                        );
                    }
                } else {
                    eprintln!("[Warning] Failed to parse config path, using default");
                }
            }

            self.obj().load_config();

            std::ops::ControlFlow::Continue(())
        }
    }

    impl GtkApplicationImpl for LupaApplication {}
    impl AdwApplicationImpl for LupaApplication {}
}

glib::wrapper! {
    pub struct LupaApplication(ObjectSubclass<imp::LupaApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl LupaApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        let app: LupaApplication = glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .property("resource-base-path", "/art/fatdawlf/Lupa")
            .build();

        app
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();

        self.add_action_entries([quit_action]);
    }

    fn load_config(&self) {
        let config_path_override = self.imp().config_path_override.get().cloned();

        let config = LupaConfig::load_config(config_path_override);
        self.imp().config.set(config).expect("Could not set config");
    }

    pub fn config(&self) -> &LupaConfig {
        self.imp().config.get().unwrap()
    }

    fn load_config_styles(&self) {
        let provider = gtk::CssProvider::new();

        let config = self.config();

        let opacity = config.aesthetic.opacity;
        let radius = config.aesthetic.radius;
        let window_color = config.aesthetic.window_color.as_str();
        let entry_elevation = config.aesthetic.entry_elevation;

        let entry_size = config.aesthetic.entry_size;
        let icon_size = entry_size as f32 * 0.75;

        provider.load_from_string(&format!(
            "
            :root {{
                --lupa-opacity: {opacity};
                --lupa-radius: {radius}px;
                --window-color: {window_color};
                --entry-elevation: {entry_elevation};
                --entry-size: {entry_size}px;
                --icon-size: {icon_size}px;
            }}
            "
        ));

        gtk::style_context_add_provider_for_display(
            &Display::default().unwrap(),
            &provider.clone(),
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    fn setup_window_config(&self, win: &LupaWindow) {
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
