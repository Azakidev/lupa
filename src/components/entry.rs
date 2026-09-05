/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use adw::{glib, prelude::*, subclass::prelude::*};
use std::cell::OnceCell;

use crate::{providers::provider::SidebarProvider, window::LupaWindow};

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "src/ui/entry.blp")]
    pub struct LupaEntry {
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub flatpak: TemplateChild<gtk::Image>,
        #[template_child]
        pub name: TemplateChild<gtk::Label>,
        #[template_child]
        pub comment: TemplateChild<gtk::Label>,

        pub provider: OnceCell<Box<dyn SidebarProvider>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LupaEntry {
        const NAME: &'static str = "LupaEntry";
        type Type = super::LupaEntry;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LupaEntry {}
    impl WidgetImpl for LupaEntry {}
    impl ButtonImpl for LupaEntry {}
}

glib::wrapper! {
    pub struct LupaEntry(ObjectSubclass<imp::LupaEntry>)
        @extends gtk::Widget, gtk::Button,
        @implements gtk::Native, gtk::Root, gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl LupaEntry {
    pub fn new<F: Fn(&LupaEntry) + 'static>(
        title: &str,
        subtitle: Option<&str>,
        icon_name: Option<&str>,
        is_app: bool,
        is_flatpak: bool,
        size: Option<u32>,
        provider: Option<Box<dyn SidebarProvider>>,
        win: &LupaWindow,
        action: F,
    ) -> Self {
        let obj: LupaEntry = glib::Object::new();

        obj.setup_appearance(title, subtitle, icon_name, is_app, is_flatpak, size);
        obj.setup_launch(action);
        obj.setup_sidebar_request(provider, win);

        obj
    }

    fn setup_appearance(
        &self,
        title: &str,
        subtitle: Option<&str>,
        icon_name: Option<&str>,
        is_app: bool,
        is_flatpak: bool,
        size: Option<u32>,
    ) {
        let imp = self.imp();
        let name = &imp.name;
        let comment = &imp.comment;
        let icon = &imp.icon;
        let flatpak = &imp.flatpak;

        name.set_text(title);

        if let Some(txt) = subtitle {
            comment.set_text(txt);
        } else {
            comment.set_visible(false);
        }

        if icon_name.is_some()
            && let Some(size) = size
        {
            if is_app {
                icon.set_from_file(icon_name);
            } else {
                icon.set_icon_name(icon_name);
            }

            icon.set_width_request(size as i32);
            icon.set_height_request(size as i32);
        } else {
            icon.set_visible(false);
        }

        if is_flatpak {
            flatpak.set_visible(true);
        }
    }

    fn setup_launch<F: Fn(&LupaEntry) + 'static>(&self, closure: F) {
        self.connect_activate(move |b| closure(b));
    }

    fn setup_sidebar_request(&self, provider: Option<Box<dyn SidebarProvider>>, win: &LupaWindow) {
        let Some(provider) = provider else {
            return;
        };

        let content = provider.populate_sidebar(self, win);

        let controller = gtk::EventControllerKey::new();

        controller.connect_key_released(glib::clone!(
            #[weak]
            win,
            #[strong]
            content,
            move |_, key, _, _| {
                let win_imp = win.imp();
                let view = &win_imp.sidebar_view;
                let sidebar = &win_imp.sidebar_content;

                if key == gtk::gdk::Key::Right {
                    sidebar.set_child(Some(&content));
                    view.set_show_sidebar(true);

                    if let Some(child) = content.imp().actions_box.first_child() {
                        child.grab_focus();
                    }
                } else {
                    sidebar.set_child(None::<&gtk::Widget>);
                    view.set_show_sidebar(false);
                }
            }
        ));

        self.add_controller(controller);
    }
}
