/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use adw::{glib, prelude::*, subclass::prelude::*};
use std::{cell::OnceCell, process::Command};

use crate::{
    providers::{
        app::{App, find_icon_path},
        provider::SidebarProvider,
    },
    utils::spawn_with_new_session,
    window::LupaWindow,
};

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

        pub app: OnceCell<App>,
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

    impl ObjectImpl for LupaEntry {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.setup_sidebar_request();
        }
    }

    impl WidgetImpl for LupaEntry {}
    impl ButtonImpl for LupaEntry {}
}

glib::wrapper! {
    pub struct LupaEntry(ObjectSubclass<imp::LupaEntry>)
        @extends gtk::Widget, gtk::Button,
        @implements gtk::Native, gtk::Root, gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl LupaEntry {
    pub fn new_app(app: App, size: u32) -> Self {
        let obj: LupaEntry = glib::Object::new();

        obj.imp().app.set(app).expect("Failed to set app");
        obj.setup_appearance_app(size);
        obj.setup_launch_app();

        obj
    }

    pub fn new_raw<F: Fn() + 'static>(
        title: &str,
        subtitle: Option<&str>,
        icon_name: Option<&str>,
        size: Option<u32>,
        action: F,
    ) -> Self {
        let obj: LupaEntry = glib::Object::new();

        obj.setup_appearance_raw(title, subtitle, icon_name, size);
        obj.setup_launch_raw(action);

        obj
    }

    fn setup_appearance_app(&self, size: u32) {
        let imp = self.imp();
        let name = &imp.name;
        let comment = &imp.comment;
        let icon = &imp.icon;
        let flatpak = &imp.flatpak;
        let app = imp.app.get().unwrap();

        name.set_text(&app.name);

        if let Some(txt) = &app.comment {
            comment.set_text(txt);
        } else {
            comment.set_visible(false);
        }

        if app.is_flatpak {
            flatpak.set_visible(true);
        }

        let icon_name = app.icon.clone().unwrap_or_default();
        let file = find_icon_path(&icon_name, size);
        icon.set_from_file(file.as_ref());
        icon.set_width_request(size as i32);
        icon.set_height_request(size as i32);
    }

    fn setup_appearance_raw(
        &self,
        title: &str,
        subtitle: Option<&str>,
        icon_name: Option<&str>,
        size: Option<u32>,
    ) {
        let imp = self.imp();
        let name = &imp.name;
        let comment = &imp.comment;
        let icon = &imp.icon;

        name.set_text(title);

        if let Some(txt) = subtitle {
            comment.set_text(txt);
        } else {
            comment.set_visible(false);
        }

        if icon_name.is_some()
            && let Some(size) = size
        {
            icon.set_icon_name(icon_name);
            icon.set_width_request(size as i32);
            icon.set_height_request(size as i32);
        } else {
            icon.set_visible(false);
        }
    }

    fn setup_launch_app(&self) {
        let imp = self.imp();
        let app = imp.app.get().unwrap();

        self.connect_activate(glib::clone!(
            #[strong]
            app,
            move |btn| {
                let raw_command: Vec<_> = app
                    .exec
                    .split_whitespace()
                    .filter(|chunk| !chunk.is_empty() && !chunk.starts_with("%"))
                    .collect();

                let [binary, args @ ..] = raw_command.as_slice() else {
                    return;
                };

                let mut command = Command::new(binary);
                command.args(args);

                if let Err(e) = spawn_with_new_session(&mut command) {
                    eprintln!("Failed to spawn process: {}", e);
                    return;
                }

                let _ = btn.activate_action("app.quit", None);
            }
        ));
    }

    fn setup_launch_raw<F: Fn() + 'static>(&self, closure: F) {
        self.connect_activate(move |_| closure());
    }

    fn setup_sidebar_request(&self) {
        let imp = self.imp();

        let Some(provider) = imp.provider.get() else {
            return;
        };

        let Some(weak) = self.ancestor(LupaWindow::static_type()) else {
            return;
        };

        let Some(win) = weak.downcast_ref::<LupaWindow>() else {
            return;
        };

        let content = provider.populate_sidebar(self);

        let controller = gtk::EventControllerKey::new();

        controller.connect_key_released(glib::clone!(
            #[weak]
            win,
            #[strong]
            content,
            move |_c, key, _, _modifier| {
                if key == gtk::gdk::Key::Right {
                    let win_imp = win.imp();
                    let sidebar = &win_imp.sidebar;

                    sidebar.set_child(Some(&content));
                }
            }
        ));
    }
}
