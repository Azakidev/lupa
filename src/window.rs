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
    application::LupaApplication,
    components::entry::LupaEntry,
    config::LupaConfig,
    providers::{
        app::AppProvider, calc::CalcProvider, emoji::EmojiProvider, file::FileProvider,
        provider::Provider, system::SystemProvider,
    },
    utils::first_visible_child,
};

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::LupaWindow)]
    #[template(file = "src/ui/window.blp")]
    pub struct LupaWindow {
        #[template_child]
        pub input: TemplateChild<gtk::Entry>,
        #[template_child]
        pub result_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub scroller: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub results: TemplateChild<gtk::Box>,
        #[template_child]
        pub sidebar_view: TemplateChild<adw::OverlaySplitView>,
        #[template_child]
        pub sidebar_content: TemplateChild<adw::Bin>,

        // Configuration entries
        #[property(get, set)]
        pub icon_size: RefCell<u32>,
        #[property(get, set)]
        pub max_file_entries: RefCell<u32>,
        #[property(get, set)]
        pub anchors: RefCell<String>,

        // Providers
        pub app_provider: AppProvider,
        pub calc_provider: CalcProvider,
        pub file_provider: FileProvider,
        pub system_provider: SystemProvider,
        pub emoji_provider: EmojiProvider,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LupaWindow {
        const NAME: &'static str = "LupaWindow";
        type Type = super::LupaWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LupaWindow {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }
        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            Self::derived_set_property(self, id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }
    }

    impl WidgetImpl for LupaWindow {}
    impl WindowImpl for LupaWindow {}
    impl ApplicationWindowImpl for LupaWindow {}
    impl AdwApplicationWindowImpl for LupaWindow {}
}

glib::wrapper! {
    pub struct LupaWindow(ObjectSubclass<imp::LupaWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gtk::Native, gtk::Root, gtk::ShortcutManager, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gio::ActionGroup, gio::ActionMap;
}

impl LupaWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P, config: &LupaConfig) -> Self {
        let icon_size = config.aesthetic.entry_size;
        let max_file_entries = config.beavior.max_file_entries;
        let anchors = config.aesthetic.anchors.clone();

        let obj: LupaWindow = glib::Object::builder()
            .property("application", application)
            .property("icon_size", icon_size)
            .property("max_file_entries", max_file_entries)
            .property("anchors", anchors)
            .build();

        obj.setup_providers();
        obj.setup_layer();
        obj.setup_watch_focus();
        obj.setup_hide_sidebar();
        obj.setup_input();

        obj
    }

    fn shrink(&self) {
        self.set_default_size(600, 48);
    }

    fn setup_layer(&self) {
        self.init_layer_shell();
        self.set_namespace(Some("lupa:launcher"));

        self.set_layer(Layer::Top);
        self.set_keyboard_mode(KeyboardMode::OnDemand);

        let margin_size = self.icon_size() as i32 / 4;

        for anchor in self.anchors().split(",") {
            match anchor.trim() {
                "top" => {
                    self.set_anchor(gtk4_layer_shell::Edge::Top, true);
                    self.set_margin(gtk4_layer_shell::Edge::Top, margin_size);
                }

                "bottom" => {
                    self.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
                    self.set_margin(gtk4_layer_shell::Edge::Bottom, margin_size);
                }
                "left" => {
                    self.set_anchor(gtk4_layer_shell::Edge::Left, true);
                    self.set_margin(gtk4_layer_shell::Edge::Left, margin_size);
                }
                "right" => {
                    self.set_anchor(gtk4_layer_shell::Edge::Right, true);
                    self.set_margin(gtk4_layer_shell::Edge::Right, margin_size);
                }
                _ => {} // No-op
            }
        }
    }

    fn setup_providers(&self) {
        glib::idle_add_local_once(glib::clone!(
            #[weak(rename_to=win)]
            self,
            move || {
                let imp = win.imp();

                imp.app_provider.prepare(&win);
                imp.calc_provider.prepare(&win);
                imp.file_provider.prepare(&win);
                imp.system_provider.prepare(&win);
                imp.emoji_provider.prepare(&win);
            }
        ));
    }

    fn setup_watch_focus(&self) {
        self.connect_is_active_notify(|win| {
            if let Some(app) = win.application().and_downcast::<LupaApplication>()
                && !win.is_active()
                && app.config().beavior.close_when_unfocused
            {
                win.close();
            }
        });
    }

    fn setup_hide_sidebar(&self) {
        let controller = gtk::EventControllerKey::new();

        controller.connect_key_released(glib::clone!(
            #[weak(rename_to=view)]
            &self.imp().sidebar_view,
            #[weak(rename_to=input)]
            &self.imp().input,
            move |_, key, _, _| {
                if key == gtk::gdk::Key::Left {
                    view.set_show_sidebar(false);

                    if let Some(child) = input.first_child()
                        && let Some(text) = child.downcast_ref::<gtk::Text>()
                        && !text.has_focus()
                    {
                        input.grab_focus();
                    }
                }
            }
        ));

        self.add_controller(controller);
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
            #[weak]
            results,
            move |i| {
                let text = i.text().trim().to_string();

                obj.imp().sidebar_view.set_show_sidebar(false);

                glib::idle_add_local_once(glib::clone!(
                    #[weak]
                    obj,
                    #[strong]
                    text,
                    move || {
                        obj.update_results(&text);

                        if text.is_empty() || first_visible_child(&results).is_none() {
                            obj.shrink();
                            revealer.set_reveal_child(false);
                        } else {
                            revealer.set_reveal_child(true);
                        }
                    }
                ));
            }
        ));

        input.connect_activate(glib::clone!(
            #[weak]
            results,
            move |_| {
                if let Some(entry) = first_visible_child(&results).and_downcast::<LupaEntry>() {
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
        imp.system_provider.hide_entries();
        imp.emoji_provider.hide_entries();
    }

    fn update_results(&self, query: &str) {
        let imp = self.imp();

        self.clear_results();

        match query {
            q if q.starts_with(AppProvider::PREFIX) => {
                imp.app_provider.update_entries(query, self);
            }
            q if q.starts_with(CalcProvider::PREFIX) => {
                imp.calc_provider.update_entries(query, self);
            }
            q if q.starts_with(FileProvider::PREFIX) => {
                imp.file_provider.update_entries(query, self);
            }
            q if q.starts_with(SystemProvider::PREFIX) => {
                imp.system_provider.update_entries(query, self);
            }
            q if q.starts_with(EmojiProvider::PREFIX) => {
                imp.emoji_provider.update_entries(query, self);
            }
            // Run all if no prefix is selected
            _ => {
                imp.system_provider.update_entries(query, self);
                imp.emoji_provider.update_entries(query, self);
                imp.app_provider.update_entries(query, self);
                imp.file_provider.update_entries(query, self);
                imp.calc_provider.update_entries(query, self);
            }
        }
    }
}
