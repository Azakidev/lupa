/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use adw::{
    gio,
    glib::{self, Properties},
    prelude::*,
    subclass::prelude::*,
};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;

use crate::{
    application::PikolaunchApplication,
    components::entry::PikolaunchEntry,
    providers::{app::AppProvider, calc::CalcProvider, file::FileProvider, provider::Provider},
    utils::first_visible_child,
};

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::PikolaunchWindow)]
    #[template(file = "src/ui/window.blp")]
    pub struct PikolaunchWindow {
        #[template_child]
        pub input: TemplateChild<gtk::Entry>,
        #[template_child]
        pub result_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub scroller: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub results: TemplateChild<gtk::Box>,

        #[property(get, set)]
        pub icon_size: RefCell<u32>,

        // Providers
        pub app_provider: AppProvider,
        pub calc_provider: CalcProvider,
        pub file_provider: FileProvider,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PikolaunchWindow {
        const NAME: &'static str = "PikolaunchWindow";
        type Type = super::PikolaunchWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PikolaunchWindow {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }
        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            Self::derived_set_property(self, id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.setup_providers();
            obj.setup_layer();
            obj.setup_watch_focus();
            obj.setup_input();
        }
    }

    impl WidgetImpl for PikolaunchWindow {}
    impl WindowImpl for PikolaunchWindow {}
    impl ApplicationWindowImpl for PikolaunchWindow {}
    impl AdwApplicationWindowImpl for PikolaunchWindow {}
}

glib::wrapper! {
    pub struct PikolaunchWindow(ObjectSubclass<imp::PikolaunchWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gtk::Native, gtk::Root, gtk::ShortcutManager, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gio::ActionGroup, gio::ActionMap;
}

impl PikolaunchWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P, icon_size: u32) -> Self {
        glib::Object::builder()
            .property("application", application)
            .property("icon_size", icon_size)
            .build()
    }

    fn shrink(&self) {
        self.set_default_size(600, 48);
    }

    fn setup_layer(&self) {
        self.init_layer_shell();
        self.set_namespace(Some("pikolaunch:launcher"));

        self.set_layer(Layer::Top);
        self.set_keyboard_mode(KeyboardMode::OnDemand);
    }

    fn setup_providers(&self) {
        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to=win)]
            self,
            async move {
                let imp = win.imp();

                imp.app_provider.prepare(&win);
                imp.calc_provider.prepare(&win);
                imp.file_provider.prepare(&win);
            }
        ));
    }

    fn setup_watch_focus(&self) {
        self.connect_is_active_notify(|win| {
            if let Some(app) = win.application().and_downcast::<PikolaunchApplication>()
                && !win.is_active()
                && app.config().beavior.close_when_unfocused
            {
                win.close();
            }
        });
    }

    fn setup_input(&self) {
        let imp = self.imp();
        let input = imp.input.get();
        let revealer = imp.result_revealer.get();
        let results = imp.results.get();

        input.connect_text_notify(glib::clone!(
            #[weak(rename_to=obj)]
            self,
            #[weak]
            revealer,
            move |i| {
                let text = i.text().trim().to_string();

                if text.is_empty() {
                    obj.shrink();
                    revealer.set_reveal_child(false);
                } else {
                    revealer.set_reveal_child(true);
                }

                obj.update_results(&text);
            }
        ));

        input.connect_activate(glib::clone!(
            #[weak]
            results,
            move |_| {
                if let Some(entry) = first_visible_child(&results).and_downcast::<PikolaunchEntry>()
                {
                    entry.activate();
                }
            }
        ));
    }

    fn clear_results(&self) {
        let imp = self.imp();

        imp.app_provider.hide_entries();
        imp.calc_provider.hide_entries();
        imp.file_provider.hide_entries();
    }

    fn update_results(&self, query: &str) {
        let imp = self.imp();

        self.clear_results();

        match query {
            q if q.starts_with(FileProvider::PREFIX) => {
                imp.file_provider.update_entries(query, self);
            }
            q if q.starts_with(CalcProvider::PREFIX) => {
                imp.calc_provider.update_entries(query, self);
            }
            q if q.starts_with(AppProvider::PREFIX) => {
                imp.app_provider.update_entries(query, self);
            }
            // Run all if no prefix is selected
            _ => {
                imp.app_provider.update_entries(query, self);
                imp.calc_provider.update_entries(query, self);
                imp.file_provider.update_entries(query, self);
            }
        }
    }
}
