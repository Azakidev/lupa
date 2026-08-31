/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use adw::{
    gio,
    glib::{self, Properties, WeakRef},
    prelude::*,
    subclass::prelude::*,
};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
};

use crate::{
    application::PikolaunchApplication, components::entry::PikolaunchEntry,
    providers::app::discover_apps,
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

        pub matcher: OnceCell<SkimMatcherV2>,
        pub cache: RefCell<HashMap<String, WeakRef<PikolaunchEntry>>>,
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
            obj.setup_layer();
            obj.setup_watch_focus();
            obj.setup_input();
            obj.setup_app_entries();
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
        let obj: PikolaunchWindow = glib::Object::builder()
            .property("application", application)
            .property("icon_size", icon_size)
            .build();

        let matcher = SkimMatcherV2::default();
        let _ = obj.imp().matcher.set(matcher);

        obj
    }

    fn setup_layer(&self) {
        self.init_layer_shell();
        self.set_namespace(Some("pikolaunch:launcher"));

        self.set_layer(Layer::Top);
        self.set_keyboard_mode(KeyboardMode::OnDemand);
    }

    fn setup_watch_focus(&self) {
        self.connect_is_active_notify(|win| {
            if let Some(app) = win.application().and_downcast::<PikolaunchApplication>()
                && !win.is_active()
                && app.config().beavior.close_when_unfocused
            {
                app.quit();
            }
        });
    }

    fn setup_input(&self) {
        let imp = self.imp();
        let input = imp.input.get();
        let revealer = imp.result_revealer.get();

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
    }

    fn shrink(&self) {
        self.set_default_size(600, 48);
    }

    fn clear_results(&self) {
        let imp = self.imp();

        imp.cache.borrow().iter().for_each(|(_, e)| {
            if let Some(entry) = e.upgrade() {
                entry.set_visible(false);
            };
        });
    }

    fn update_results(&self, query: &str) {
        let imp = self.imp();
        let cache = imp.cache.borrow();
        let results = imp.results.get();
        let matcher = imp.matcher.get().unwrap();

        self.clear_results();

        let mut filtered = cache
            .iter()
            .filter(|(a, _)| {
                query
                    .to_lowercase()
                    .chars()
                    .all(|c| a.to_lowercase().contains(&c.to_string()))
            })
            .map(|(a, _)| a.clone())
            .collect::<Vec<String>>();

        filtered.sort_unstable_by_key(|a| {
            matcher.fuzzy_match(&a.to_lowercase(), &query.to_lowercase())
        });

        let mut prev: Option<WeakRef<PikolaunchEntry>> = None;

        for a in filtered.iter().rev() {
            if let Some(weak) = cache.get(a)
                && let Some(entry) = weak.upgrade()
            {
                if let Some(prev_weak) = prev {
                    results.reorder_child_after(&entry, prev_weak.upgrade().as_ref());
                }

                entry.set_visible(true);
                prev = Some(entry.downgrade());
            }
        }
    }

    fn setup_app_entries(&self) {
        let weak = self.downgrade();
        glib::spawn_future_local(async move {
            let Some(obj) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };

            let imp = obj.imp();
            let mut cache = imp.cache.borrow_mut();
            let results = &imp.results;

            let icon_size = obj.icon_size();

            let apps = discover_apps().unwrap_or_default();

            for app in apps {
                let entry = PikolaunchEntry::new(app.clone(), icon_size);
                results.append(&entry);

                cache.insert(app.name, entry.downgrade());
            }

            glib::ControlFlow::Break
        });
    }
}
