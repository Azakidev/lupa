/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use adw::{glib, prelude::*, subclass::prelude::*};

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(file = "src/ui/sidebar.blp")]
    pub struct LupaSidebarContent {
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        #[template_child]
        pub subtitle: TemplateChild<gtk::Label>,
        #[template_child]
        pub actions_box: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LupaSidebarContent {
        const NAME: &'static str = "LupaSidebarContent";
        type Type = super::LupaSidebarContent;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LupaSidebarContent {}
    impl WidgetImpl for LupaSidebarContent {}
    impl BoxImpl for LupaSidebarContent {}
}

glib::wrapper! {
    pub struct LupaSidebarContent(ObjectSubclass<imp::LupaSidebarContent>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Native, gtk::Root, gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl LupaSidebarContent {
    pub fn new(
        title: &str,
        subtitle: Option<&str>,
        icon: Option<&str>,
        icon_size: u32,
        icon_is_file: bool,
    ) -> Self {
        let obj: LupaSidebarContent = glib::Object::new();

        obj.setup_appearance(title, subtitle, icon, icon_size, icon_is_file);

        obj
    }

    fn setup_appearance(
        &self,
        title: &str,
        subtitle: Option<&str>,
        icon: Option<&str>,
        icon_size: u32,
        icon_is_file: bool,
    ) {
        let imp = self.imp();

        let icon_image = &imp.icon;
        let title_label = &imp.title;
        let subtitle_label = &imp.subtitle;

        if let Some(icon) = icon {
            icon_image.set_pixel_size(icon_size as i32);
            if icon_is_file {
                icon_image.set_from_file(Some(icon));
            } else {
                icon_image.set_icon_name(Some(icon));
            }
        } else {
            icon_image.set_visible(false);
        }

        title_label.set_text(title);

        if let Some(sub) = subtitle {
            subtitle_label.set_text(sub);
        } else {
            subtitle_label.set_visible(false);
        }
    }

    pub fn add_action<F: Fn(&gtk::Button) + 'static>(
        &self,
        name: &str,
        icon: Option<&str>,
        action: F,
    ) {
        let content = adw::ButtonContent::builder()
            .halign(gtk::Align::Start)
            .margin_start(14)
            .margin_top(12)
            .margin_bottom(12)
            .name(name)
            .label(name);

        let content = if let Some(icon) = icon {
            content.icon_name(icon)
        } else {
            content
        };

        let content = content.build();

        let button = gtk::Button::builder()
            .css_classes(["card", "launcher-entry"])
            .hexpand(true)
            .child(&content)
            .build();

        button.connect_activate(move |b| action(b));

        self.imp().actions_box.append(&button);
    }
}
