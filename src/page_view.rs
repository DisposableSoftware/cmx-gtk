//! Draws one or two pages fitted to the widget (`WIN-6`).

use crate::layout;
use gtk::{gdk, glib, graphene, gsk, prelude::*, subclass::prelude::*};

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct PageView {
        pub pages: RefCell<Vec<gdk::Texture>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PageView {
        const NAME: &'static str = "CmxPageView";
        type Type = super::PageView;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for PageView {}

    impl WidgetImpl for PageView {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let area = (widget.width() as f64, widget.height() as f64);
            let pages = self.pages.borrow();
            let sizes: Vec<(f64, f64)> = pages
                .iter()
                .map(|t| (t.width() as f64, t.height() as f64))
                .collect();
            for (texture, r) in pages.iter().zip(layout::place(area, &sizes)) {
                let bounds =
                    graphene::Rect::new(r.x as f32, r.y as f32, r.width as f32, r.height as f32);
                // Trilinear uses mipmaps, so a big page shrinks without aliasing.
                snapshot.append_scaled_texture(texture, gsk::ScalingFilter::Trilinear, &bounds);
            }
        }
    }
}

glib::wrapper! {
    pub struct PageView(ObjectSubclass<imp::PageView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for PageView {
    fn default() -> Self {
        glib::Object::builder()
            .property("hexpand", true)
            .property("vexpand", true)
            .build()
    }
}

impl PageView {
    pub fn set_pages(&self, pages: Vec<gdk::Texture>) {
        *self.imp().pages.borrow_mut() = pages;
        self.queue_draw();
    }

    /// Width and height of the first page shown.
    pub fn first_size(&self) -> Option<(i32, i32)> {
        let pages = self.imp().pages.borrow();
        pages.first().map(|t| (t.width(), t.height()))
    }
}
