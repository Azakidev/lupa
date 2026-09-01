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
use fuzzy_matcher::skim::SkimMatcherV2;
use gtk::glib::property::PropertySet;
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
};

use crate::{
    application::PikolaunchApplication,
    components::entry::PikolaunchEntry,
    providers::{
        app::{discover_apps, update_app_results},
        calc::update_calc_results,
        file::update_file_search_results,
    },
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

        pub matcher: OnceCell<SkimMatcherV2>,
        pub cache: RefCell<HashMap<String, WeakRef<PikolaunchEntry>>>,

        pub calc_entry: RefCell<Option<WeakRef<PikolaunchEntry>>>,
        pub files_cache: RefCell<HashMap<String, WeakRef<PikolaunchEntry>>>,
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
        obj.imp().files_cache.set(HashMap::new());

        obj
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
                let entry = PikolaunchEntry::new_app(app.clone(), icon_size);
                results.append(&entry);

                cache.insert(app.name, entry.downgrade());
            }

            glib::ControlFlow::Break
        });
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
        let results = &imp.results;
        let calc_entry = imp.calc_entry.borrow();

        if let Some(weak) = calc_entry.as_ref()
            && let Some(entry) = weak.upgrade()
        {
            results.remove(&entry);
        };

        imp.cache.borrow().iter().for_each(|(_, e)| {
            if let Some(entry) = e.upgrade() {
                entry.set_visible(false);
            };
        });
    }

    fn update_results(&self, query: &str) {
        let imp = self.imp();
        let cache = imp.cache.borrow();
        let mut file_cache = imp.files_cache.borrow_mut();
        let calc_entry = &imp.calc_entry;
        let results = imp.results.get();
        let matcher = imp.matcher.get().unwrap();

        self.clear_results();

        match query {
            q if q.starts_with("/") => {
                update_file_search_results(
                    q.strip_prefix("/").unwrap(),
                    &mut file_cache,
                    self,
                    &results,
                    self.icon_size(),
                );
            }
            q if q.starts_with("=") => {
                if let Some(entry) =
                    update_calc_results(query.strip_prefix("=").unwrap(), self.icon_size(), self)
                {
                    results.prepend(&entry);
                    calc_entry.replace(Some(entry.downgrade()));
                };
            }
            q if q.starts_with("#") => {
                // Explicit to applications
                update_app_results(q.strip_prefix("#").unwrap(), &cache, &results, matcher);
            }
            _ => {
                update_app_results(query, &cache, &results, matcher);

                if let Some(entry) = update_calc_results(
                    query.strip_prefix("=").unwrap_or(query),
                    self.icon_size(),
                    self,
                ) {
                    results.prepend(&entry);
                    calc_entry.replace(Some(entry.downgrade()));
                };
            }
        }
    }
}
