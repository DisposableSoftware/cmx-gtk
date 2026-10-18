//! Reading progress and the last comic (`READ-40`, `READ-41`), in
//! `$XDG_STATE_HOME/cmx/state.ini`.

use gtk::{gio, glib};
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const PAGES: &str = "pages";
const SESSION: &str = "session";
const LAST: &str = "last";

pub fn file() -> PathBuf {
    glib::user_state_dir().join("cmx").join("state.ini")
}

pub struct Progress {
    path: PathBuf,
    kf: glib::KeyFile,
    writing: Cell<bool>,
    dirty: Cell<bool>,
}

thread_local! {
    static PROGRESS: Rc<Progress> = Rc::new(Progress::load(file()));
}

pub fn get() -> Rc<Progress> {
    PROGRESS.with(Rc::clone)
}

/// Key-file keys can't hold every path character, so paths are URI-escaped.
fn key(path: &Path) -> String {
    glib::Uri::escape_string(&path.to_string_lossy(), Some("/"), false).into()
}

impl Progress {
    /// Reads the file once at startup. A missing or broken file is no progress.
    pub fn load(path: PathBuf) -> Self {
        let kf = glib::KeyFile::new();
        if let Err(e) = kf.load_from_file(&path, glib::KeyFileFlags::NONE)
            && !e.matches(glib::FileError::Noent)
        {
            glib::g_warning!("cmx", "{}: {}", path.display(), e.message());
        }
        Self {
            path,
            kf,
            writing: Cell::new(false),
            dirty: Cell::new(false),
        }
    }

    pub fn page(&self, comic: &Path) -> Option<usize> {
        let page = self.kf.uint64(PAGES, &key(comic)).ok()?;
        usize::try_from(page).ok()
    }

    pub fn set_page(self: &Rc<Self>, comic: &Path, page: usize) {
        self.kf.set_uint64(PAGES, &key(comic), page as u64);
        self.save();
    }

    /// The last comic that opened. Whether it still exists is checked when it is
    /// opened, off the main thread.
    pub fn last(&self) -> Option<PathBuf> {
        self.kf
            .string(SESSION, LAST)
            .ok()
            .map(|s| PathBuf::from(s.as_str()))
    }

    pub fn set_last(self: &Rc<Self>, comic: &Path) {
        self.kf.set_string(SESSION, LAST, &comic.to_string_lossy());
        self.save();
    }

    /// Writes on a worker thread, atomically; saves made meanwhile are folded into
    /// one more write.
    fn save(self: &Rc<Self>) {
        self.dirty.set(true);
        if self.writing.replace(true) {
            return;
        }
        let this = self.clone();
        glib::spawn_future_local(async move {
            while this.dirty.replace(false) {
                let data = this.kf.to_data().to_string();
                let path = this.path.clone();
                let written = gio::spawn_blocking(move || write(&path, &data)).await;
                if let Ok(Err(e)) = written {
                    glib::g_warning!("cmx", "{}: {e}", this.path.display());
                }
            }
            this.writing.set(false);
        });
    }
}

fn write(path: &Path, data: &str) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    glib::file_set_contents(path, data.as_bytes()).map_err(|e| e.message().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::tests::{run, settle};

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("cmx-test-progress-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("cmx").join("state.ini")
    }

    #[test]
    fn round_trips_pages_and_the_last_comic() {
        let path = temp("trip");
        let odd = Path::new("/comics/a=b [c] #1 é.cbz");
        run(async {
            let p = Rc::new(Progress::load(path.clone()));
            assert_eq!(p.page(odd), None);
            assert_eq!(p.last(), None);
            p.set_page(odd, 3);
            p.set_page(odd, 7);
            p.set_page(Path::new("/comics/other.cbz"), 1);
            p.set_last(odd);
            settle().await;
        });
        let p = Progress::load(path.clone());
        assert_eq!(p.page(odd), Some(7));
        assert_eq!(p.page(Path::new("/comics/other.cbz")), Some(1));
        assert_eq!(p.page(Path::new("/comics/moved.cbz")), None);
        assert_eq!(p.last().as_deref(), Some(odd));
        std::fs::remove_dir_all(path.parent().unwrap().parent().unwrap()).unwrap();
    }

    #[test]
    fn a_broken_file_is_no_progress() {
        let path = temp("broken");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "[pages\nnot a key file").unwrap();
        let p = Progress::load(path.clone());
        assert_eq!(p.last(), None);
        assert_eq!(p.page(Path::new("/x.cbz")), None);
        std::fs::remove_dir_all(path.parent().unwrap().parent().unwrap()).unwrap();
    }
}
