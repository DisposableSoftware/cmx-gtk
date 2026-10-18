//! A reader window: one comic, its pages, and the win.* actions.

use crate::archive::{Archive, Error, file_name};
use crate::page_view::PageView;
use crate::reader::{self, Book};
use crate::session::{Decoder, Fetch, Session};
use crate::{config, decode, fit, info, progress};
use gtk::{gdk, gio, glib, prelude::*};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

/// Actions that mean nothing without a comic (`WIN-14`).
const COMIC_ACTIONS: &[&str] = &[
    "next-page",
    "previous-page",
    "first-page",
    "last-page",
    "toggle-dual-page",
    "toggle-info",
];

struct Comic {
    book: Book,
    session: Session<gdk::Texture>,
}

#[derive(Default)]
struct State {
    comic: Option<Rc<Comic>>,
    /// The file being opened, so opening it again finds this window (`WIN-9`).
    opening: Option<PathBuf>,
    index: usize,
    /// Pages on screen: 1, or 2 for a spread.
    shown: usize,
    dual: bool,
    /// Bumped on every page change, so a late decode can tell it is stale.
    generation: u64,
}

pub struct Win {
    pub window: gtk::ApplicationWindow,
    stack: gtk::Stack,
    view: PageView,
    info: gtk::ScrolledWindow,
    state: RefCell<State>,
}

thread_local! {
    static WINDOWS: RefCell<Vec<Rc<Win>>> = const { RefCell::new(Vec::new()) };
}

fn windows() -> Vec<Rc<Win>> {
    WINDOWS.with(|w| w.borrow().clone())
}

/// Opens `files`, each in its own window (`WIN-8`, `WIN-9`). `origin` is the window
/// the files came from; the first goes there if it is empty.
pub async fn open_files(app: gtk::Application, origin: Option<Rc<Win>>, files: Vec<gio::File>) {
    let mut origin = origin;
    for file in files {
        let Some(path) = file.path() else {
            glib::g_warning!("cmx", "{}: not a local file", file.uri());
            continue;
        };
        let Ok(path) =
            gio::spawn_blocking(move || std::fs::canonicalize(&path).unwrap_or(path)).await
        else {
            continue;
        };
        if let Some(open) = windows().into_iter().find(|w| w.has(&path)) {
            open.window.present();
            continue;
        }
        let win = match origin.take() {
            Some(w) if w.is_empty() => w,
            _ => Win::new(&app),
        };
        win.load(path, false);
    }
}

fn is_cbz(file: &gio::File) -> bool {
    file.basename()
        .and_then(|n| n.extension().map(|e| e.eq_ignore_ascii_case("cbz")))
        .unwrap_or(false)
}

/// The placeholder's second line (`WIN-5`).
fn hint(app: &gtk::Application) -> String {
    let first = app.accels_for_action("win.open").into_iter().next();
    match first.and_then(|a| gtk::accelerator_parse(a.as_str())) {
        Some((key, mods)) => format!(
            "{}, or drop a .cbz here",
            gtk::accelerator_get_label(key, mods)
        ),
        None => "Drop a .cbz here".into(),
    }
}

impl Win {
    pub fn new(app: &gtk::Application) -> Rc<Self> {
        let cfg = config::get();
        let placeholder = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();
        placeholder.append(
            &gtk::Label::builder()
                .label("Open a comic")
                .css_classes(["placeholder-title"])
                .build(),
        );
        placeholder.append(
            &gtk::Label::builder()
                .label(hint(app))
                .css_classes(["placeholder-hint"])
                .build(),
        );
        let spinner = gtk::Spinner::builder()
            .spinning(true)
            .width_request(32)
            .height_request(32)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();
        let view = PageView::default();
        let stack = gtk::Stack::builder()
            .hexpand(true)
            .vexpand(true)
            .css_classes(["page-area"])
            .build();
        stack.add_named(&placeholder, Some("empty"));
        stack.add_named(&spinner, Some("opening"));
        stack.add_named(&view, Some("pages"));
        let info = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .width_request(260)
            // The value labels expand, and that would spread up to the panel.
            .hexpand(false)
            .visible(false)
            .css_classes(["info-panel"])
            .build();
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        row.append(&stack);
        row.append(&info);
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("cmx")
            .default_width(cfg.width)
            .default_height(cfg.height)
            .child(&row)
            .build();
        window.add_css_class("cmx");

        let this = Rc::new(Self {
            window,
            stack,
            view,
            info,
            state: RefCell::default(),
        });
        this.add_actions();
        this.add_drop_target(app);
        this.set_comic_actions(false);
        WINDOWS.with(|w| w.borrow_mut().push(this.clone()));
        this.window.connect_close_request(|window| {
            WINDOWS.with(|w| w.borrow_mut().retain(|x| &x.window != window));
            glib::Propagation::Proceed
        });
        this.window.present();
        this
    }

    fn add_actions(self: &Rc<Self>) {
        type Handler = fn(&Rc<Win>);
        let entries: [(&str, Handler); 10] = [
            ("open", |w| w.choose_files()),
            ("new-window", |w| {
                if let Some(app) = w.window.application() {
                    Win::new(&app);
                }
            }),
            ("next-page", |w| {
                w.go(|st, count| reader::next(st.index, st.shown, count))
            }),
            ("previous-page", |w| {
                w.go(|st, _| reader::previous(st.index, st.dual))
            }),
            ("first-page", |w| w.go(|st, _| reader::first(st.index))),
            ("last-page", |w| {
                w.go(|st, count| reader::last(st.index, count))
            }),
            ("toggle-dual-page", |w| w.toggle_dual()),
            ("toggle-info", |w| w.info.set_visible(!w.info.is_visible())),
            ("toggle-fullscreen", |w| {
                w.window.set_fullscreened(!w.window.is_fullscreen())
            }),
            ("exit-fullscreen", |w| {
                if w.window.is_fullscreen() {
                    w.window.unfullscreen();
                }
            }),
        ];
        let weak = Rc::downgrade(self);
        let actions: Vec<_> = entries
            .into_iter()
            .map(|(name, handler)| {
                let weak = weak.clone();
                gio::ActionEntry::builder(name)
                    .activate(move |_: &gtk::ApplicationWindow, _, _| {
                        if let Some(w) = weak.upgrade() {
                            handler(&w);
                        }
                    })
                    .build()
            })
            .collect();
        self.window.add_action_entries(actions);
    }

    fn add_drop_target(self: &Rc<Self>, app: &gtk::Application) {
        let target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
        let weak = Rc::downgrade(self);
        let app = app.downgrade();
        target.connect_drop(move |_, value, _, _| {
            let (Some(win), Some(app)) = (weak.upgrade(), app.upgrade()) else {
                return false;
            };
            let Ok(list) = value.get::<gdk::FileList>() else {
                return false;
            };
            let files: Vec<gio::File> = list.files().into_iter().filter(is_cbz).collect();
            if files.is_empty() {
                return false;
            }
            glib::spawn_future_local(open_files(app, Some(win), files));
            true
        });
        self.window.add_controller(target);
    }

    fn set_comic_actions(&self, enabled: bool) {
        for name in COMIC_ACTIONS {
            if let Some(action) = self
                .window
                .lookup_action(name)
                .and_downcast::<gio::SimpleAction>()
            {
                action.set_enabled(enabled);
            }
        }
    }

    fn is_empty(&self) -> bool {
        let st = self.state.borrow();
        st.comic.is_none() && st.opening.is_none()
    }

    fn has(&self, path: &Path) -> bool {
        let st = self.state.borrow();
        st.opening.as_deref() == Some(path)
            || st.comic.as_ref().is_some_and(|c| c.book.path == path)
    }

    fn choose_files(self: &Rc<Self>) {
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Comics"));
        filter.add_mime_type("application/vnd.comicbook+zip");
        filter.add_suffix("cbz");
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        let dialog = gtk::FileDialog::builder()
            .title("Open a comic")
            .modal(true)
            .filters(&filters)
            .default_filter(&filter)
            .build();
        let this = self.clone();
        glib::spawn_future_local(async move {
            let Ok(list) = dialog.open_multiple_future(Some(&this.window)).await else {
                return;
            };
            let files = list.iter::<gio::File>().filter_map(Result::ok).collect();
            if let Some(app) = this.window.application() {
                open_files(app, Some(this), files).await;
            }
        });
    }

    /// Opens `path` here. `restoring` is the last-comic case (`READ-41`), where a file
    /// that has gone away just leaves the placeholder.
    pub fn load(self: &Rc<Self>, path: PathBuf, restoring: bool) {
        self.state.borrow_mut().opening = Some(path.clone());
        self.stack.set_visible_child_name("opening");
        let this = self.clone();
        glib::spawn_future_local(async move {
            let p = path.clone();
            let opened = gio::spawn_blocking(move || {
                if restoring && !p.exists() {
                    return None;
                }
                Some(reader::open(&p))
            })
            .await;
            this.state.borrow_mut().opening = None;
            match opened {
                Ok(Some(Ok((book, archive)))) => this.show_comic(book, archive).await,
                Ok(None) => this.stack.set_visible_child_name("empty"),
                Ok(Some(Err(e))) => {
                    this.stack.set_visible_child_name("empty");
                    this.alert(&e.to_string());
                }
                Err(_) => {
                    this.stack.set_visible_child_name("empty");
                    let reason = "the reader crashed".into();
                    let name = file_name(&path);
                    this.alert(&Error::Open { name, reason }.to_string());
                }
            }
        });
    }

    async fn show_comic(self: &Rc<Self>, book: Book, archive: Archive) {
        let archive = Arc::new(archive);
        let entries = Arc::new(book.pages.clone());
        let decoder: Decoder<gdk::Texture> = Arc::new(move |i, _| {
            let entry = &entries[i];
            let undecodable = || Error::Undecodable(entry.clone()).to_string();
            let data = archive.read(entry).map_err(|_| undecodable())?;
            decode::texture(entry, data).ok_or_else(undecodable)
        });
        let progress = progress::get();
        let index = reader::resume(progress.page(&book.path), book.pages.len());
        progress.set_last(&book.path);
        gtk::RecentManager::default().add_item(&gio::File::for_path(&book.path).uri());
        self.info
            .set_child(Some(&info::panel(book.metadata.as_ref())));
        let session = Session::new(book.pages.len(), decoder);
        {
            let mut st = self.state.borrow_mut();
            st.comic = Some(Rc::new(Comic { book, session }));
            st.index = index;
            st.shown = 1;
        }
        self.set_comic_actions(true);
        self.update_title();
        self.show_page(index).await;
        self.fit_to_page();
    }

    fn go(self: &Rc<Self>, target: impl FnOnce(&State, usize) -> Option<usize>) {
        let (index, path) = {
            let st = self.state.borrow();
            let Some(comic) = &st.comic else { return };
            let Some(index) = target(&st, comic.book.pages.len()) else {
                return;
            };
            (index, comic.book.path.clone())
        };
        self.state.borrow_mut().index = index;
        progress::get().set_page(&path, index);
        let this = self.clone();
        glib::spawn_future_local(async move { this.show_page(index).await });
    }

    fn toggle_dual(self: &Rc<Self>) {
        let index = {
            let mut st = self.state.borrow_mut();
            st.dual = !st.dual;
            st.index
        };
        let this = self.clone();
        glib::spawn_future_local(async move { this.show_page(index).await });
    }

    async fn show_page(self: &Rc<Self>, index: usize) {
        let (comic, dual, generation) = {
            let mut st = self.state.borrow_mut();
            let Some(comic) = st.comic.clone() else {
                return;
            };
            st.generation += 1;
            (comic, st.dual, st.generation)
        };
        let result = reader::spread(&comic.session, index, dual).await;
        if self.state.borrow().generation != generation {
            return;
        }
        let shown = match result {
            Ok((page, partner)) => {
                let shown = if partner.is_some() { 2 } else { 1 };
                self.view
                    .set_pages([page].into_iter().chain(partner).collect());
                shown
            }
            Err(Fetch::Cancelled) => return,
            Err(Fetch::Failed(message)) => {
                self.view.set_pages(Vec::new());
                self.alert(&message);
                1
            }
        };
        self.state.borrow_mut().shown = shown;
        self.stack.set_visible_child_name("pages");
        self.update_title();
        comic.session.prefetch(index);
    }

    fn update_title(&self) {
        let st = self.state.borrow();
        let title = reader::window_title(st.comic.as_ref().map(|c| (&c.book, st.index, st.shown)));
        self.window.set_title(Some(&title));
    }

    /// `WIN-11`: once per open, fit a floating window to the page shown.
    fn fit_to_page(&self) {
        let w = &self.window;
        if w.is_maximized() || w.is_fullscreen() {
            return;
        }
        let Some(surface) = w.surface() else { return };
        if let Some(toplevel) = surface.downcast_ref::<gdk::Toplevel>() {
            let tiled = gdk::ToplevelState::TILED
                | gdk::ToplevelState::TOP_TILED
                | gdk::ToplevelState::BOTTOM_TILED
                | gdk::ToplevelState::LEFT_TILED
                | gdk::ToplevelState::RIGHT_TILED;
            if toplevel.state().intersects(tiled) {
                return;
            }
        }
        let Some(monitor) = WidgetExt::display(w).monitor_at_surface(&surface) else {
            return;
        };
        let Some((pw, ph)) = self.view.first_size() else {
            return;
        };
        let geometry = monitor.geometry();
        let (width, height) = w.default_size();
        let chrome = (height - self.stack.height()).max(0);
        let side = (width - self.stack.width()).max(0);
        let cfg = config::get();
        let never_sized = (width, height) == (cfg.width, cfg.height);
        let Some((cw, ch)) = fit::content_size(
            f64::from(pw) / f64::from(ph),
            f64::from(height - chrome),
            never_sized,
            f64::from(chrome),
            (f64::from(geometry.width()), f64::from(geometry.height())),
        ) else {
            return;
        };
        w.set_default_size(cw + side, ch + chrome);
    }

    fn alert(&self, message: &str) {
        gtk::AlertDialog::builder()
            .message(message)
            .buttons(["OK"])
            .modal(true)
            .build()
            .show(Some(&self.window));
    }
}
