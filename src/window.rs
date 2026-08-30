/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use adw::{gio, glib, prelude::*, subclass::prelude::*};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};

use crate::{application::PikolaunchApplication, providers::app::{App, find_icon_path}};

mod imp {

    use std::cell::OnceCell;

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
            if let Some(app) = win.application().and_downcast::<PikolaunchApplication>() {
                if !win.is_active() && app.config().beavior.close_when_unfocused {
                    app.quit();
                }
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

                if text.len() == 0 {
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
        let results = imp.results.get();

        while let Some(child) = results.first_child() {
            results.remove(&child);
        }
    }

    fn update_results(&self, query: &str) {
        let imp = self.imp();
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
                .map(|a| a.clone())
                .collect::<Vec<App>>();

            filtered.sort_unstable_by_key(|a| {
                matcher.fuzzy_match(&a.name.to_lowercase(), &query.to_lowercase())
            });

            filtered.iter().rev().for_each(|a| {
                let name = a.icon.clone().unwrap_or_default();
                let file = find_icon_path(&name);
                let image = gtk::Image::from_file(file.unwrap_or_default());
                let label = gtk::Label::new(Some(&a.name));

                let cont = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                cont.set_hexpand(true);
                cont.append(&image);
                cont.append(&label);

                results.append(&cont);
            });
        }
    }
}
