/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use adw::{
    gio,
    glib::{self, WeakRef},
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
    application::PikolaunchApplication, components::entry::PikolaunchEntry, providers::app::App,
};

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
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
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        let obj: PikolaunchWindow = glib::Object::builder()
            .property("application", application)
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
                let text = i.text().to_string();

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

        if let Some(app) = self.application().and_downcast::<PikolaunchApplication>() {
            let apps = app.apps();

            let mut filtered = apps
                .iter()
                .filter(|a| {
                    query
                        .to_lowercase()
                        .chars()
                        .all(|c| a.name.to_lowercase().contains(&c.to_string()))
                })
                .cloned()
                .collect::<Vec<App>>();

            filtered.sort_unstable_by_key(|a| {
                matcher.fuzzy_match(&a.name.to_lowercase(), &query.to_lowercase())
            });

            let mut prev: Option<WeakRef<PikolaunchEntry>> = None;

            for a in filtered.iter().rev() {
                if let Some(weak) = cache.get(&a.name)
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

            if let Some(app) = obj.application().and_downcast::<PikolaunchApplication>() {
                let apps = app.apps();

                for app in apps {
                    let entry = PikolaunchEntry::new(app.clone());
                    results.append(&entry);

                    cache.insert(app.name.clone(), entry.downgrade());
                }
            }

            glib::ControlFlow::Break
        });
    }
}
