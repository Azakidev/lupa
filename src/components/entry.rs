/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use adw::{glib, prelude::*, subclass::prelude::*};
use std::{
    cell::OnceCell,
    io,
    os::unix::process::CommandExt,
    process::{Child, Command, Stdio},
};

use crate::providers::app::{App, find_icon_path};

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "src/ui/entry.blp")]
    pub struct PikolaunchEntry {
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub flatpak: TemplateChild<gtk::Image>,
        #[template_child]
        pub name: TemplateChild<gtk::Label>,
        #[template_child]
        pub comment: TemplateChild<gtk::Label>,

        pub app: OnceCell<App>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PikolaunchEntry {
        const NAME: &'static str = "PikolaunchEntry";
        type Type = super::PikolaunchEntry;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PikolaunchEntry {
        fn constructed(&self) {
            self.parent_constructed();

            let _obj = self.obj();
        }
    }

    impl WidgetImpl for PikolaunchEntry {}
    impl ButtonImpl for PikolaunchEntry {}
}

glib::wrapper! {
    pub struct PikolaunchEntry(ObjectSubclass<imp::PikolaunchEntry>)
        @extends gtk::Widget, gtk::Button,
        @implements gtk::Native, gtk::Root, gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl PikolaunchEntry {
    pub fn new(app: App) -> Self {
        let obj: PikolaunchEntry = glib::Object::new();

        obj.imp().app.set(app).expect("Failed to set app");
        obj.setup_appearance();
        obj.setup_launch();

        obj
    }

    fn setup_appearance(&self) {
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
        let file = find_icon_path(&icon_name);
        icon.set_from_file(file.as_ref());
    }

    fn setup_launch(&self) {
        let imp = self.imp();
        let app = imp.app.get().unwrap();

        self.connect_activate(glib::clone!(
            #[strong]
            app,
            move |b| {
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

                let _ = b.activate_action("app.quit", None);
            }
        ));
    }
}

fn spawn_with_new_session(command: &mut Command) -> io::Result<Child> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // SAFETY: We are in the "fork-exec gap".
    // We avoid heap allocation and use only async-signal-safe calls.
    unsafe {
        command.pre_exec(|| {
            nix::unistd::setsid()
                .map(|_| ())
                .map_err(|e| io::Error::from_raw_os_error(e as i32))
        });
    }

    command.spawn()
}
