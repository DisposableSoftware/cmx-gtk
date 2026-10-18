//! Decoded pages for one open comic (`READ-21`): decodes run on worker threads, a
//! small cache is kept around the current page, and decodes the user has paged past
//! are cancelled. Lives on the main thread.

use gtk::{gio, glib};
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};

/// Decoded pages farther than this from the current one are dropped.
pub const KEEP_RADIUS: usize = 2;

pub trait PageImage: Clone + Send + 'static {
    fn size(&self) -> (i32, i32);

    /// Landscape: already a two-page spread, so never paired (`READ-30`).
    fn is_wide(&self) -> bool {
        let (w, h) = self.size();
        h > 0 && w > h
    }
}

impl PageImage for gtk::gdk::Texture {
    fn size(&self) -> (i32, i32) {
        use gtk::prelude::*;
        (self.width(), self.height())
    }
}

/// Decodes page `index`; the flag turns true when nobody wants the result any more.
pub type Decoder<T> = Arc<dyn Fn(usize, &AtomicBool) -> Result<T, String> + Send + Sync>;

#[derive(Debug, Clone, PartialEq)]
pub enum Fetch {
    /// Paged past before it finished; nothing to report.
    Cancelled,
    Failed(String),
}

enum Slot<T> {
    Pending {
        cancel: Arc<AtomicBool>,
        waiters: Vec<Waker>,
    },
    Ready(Result<T, String>),
}

struct Inner<T> {
    decoder: Decoder<T>,
    count: usize,
    focus: usize,
    slots: HashMap<usize, Slot<T>>,
}

pub struct Session<T> {
    inner: Rc<RefCell<Inner<T>>>,
}

impl<T> Clone for Session<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: PageImage> Session<T> {
    pub fn new(count: usize, decoder: Decoder<T>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                decoder,
                count,
                focus: 0,
                slots: HashMap::new(),
            })),
        }
    }

    pub fn count(&self) -> usize {
        self.inner.borrow().count
    }

    /// Page `index`. `refocus: false` is for the right-hand page of a spread, so
    /// fetching it doesn't move the cache window (`READ-32`).
    pub async fn image(&self, index: usize, refocus: bool) -> Result<T, Fetch> {
        if index >= self.count() {
            return Err(Fetch::Failed("That page does not exist.".into()));
        }
        if refocus {
            self.focus_on(index);
        }
        self.ensure(index);
        Wait {
            inner: self.inner.clone(),
            index,
        }
        .await
    }

    pub fn prefetch(&self, index: usize) {
        for neighbour in [index.checked_sub(1), Some(index + 1)]
            .into_iter()
            .flatten()
        {
            if neighbour < self.count() {
                self.ensure(neighbour);
            }
        }
    }

    /// The indexes decoded or decoding, for tests.
    #[cfg(test)]
    pub fn held(&self) -> Vec<usize> {
        let mut v: Vec<_> = self.inner.borrow().slots.keys().copied().collect();
        v.sort();
        v
    }

    fn focus_on(&self, index: usize) {
        let mut waiters = Vec::new();
        {
            let mut inner = self.inner.borrow_mut();
            inner.focus = index;
            inner.slots.retain(|&i, slot| {
                let keep = i.abs_diff(index) <= KEEP_RADIUS;
                if !keep && let Slot::Pending { cancel, waiters: w } = slot {
                    cancel.store(true, Ordering::Relaxed);
                    waiters.append(w);
                }
                keep
            });
        }
        waiters.into_iter().for_each(Waker::wake);
    }

    fn ensure(&self, index: usize) {
        let (decoder, cancel) = {
            let mut inner = self.inner.borrow_mut();
            if inner.slots.contains_key(&index) {
                return;
            }
            let cancel = Arc::new(AtomicBool::new(false));
            inner.slots.insert(
                index,
                Slot::Pending {
                    cancel: cancel.clone(),
                    waiters: Vec::new(),
                },
            );
            (inner.decoder.clone(), cancel)
        };
        let weak = Rc::downgrade(&self.inner);
        glib::MainContext::ref_thread_default().spawn_local(async move {
            let flag = cancel.clone();
            let result = gio::spawn_blocking(move || {
                if flag.load(Ordering::Relaxed) {
                    return None;
                }
                let result = decoder(index, &flag);
                (!flag.load(Ordering::Relaxed)).then_some(result)
            })
            .await
            .unwrap_or_else(|_| Some(Err("the decoder crashed".into())));
            finish(&weak, index, &cancel, result);
        });
    }
}

fn finish<T>(
    inner: &Weak<RefCell<Inner<T>>>,
    index: usize,
    cancel: &Arc<AtomicBool>,
    result: Option<Result<T, String>>,
) {
    let Some(inner) = inner.upgrade() else {
        return;
    };
    let waiters = {
        let mut inner = inner.borrow_mut();
        let focus = inner.focus;
        let Some(Slot::Pending { cancel: c, waiters }) = inner.slots.get_mut(&index) else {
            return;
        };
        if !Arc::ptr_eq(c, cancel) {
            return;
        }
        let waiters = std::mem::take(waiters);
        match result {
            Some(result) if index.abs_diff(focus) <= KEEP_RADIUS => {
                inner.slots.insert(index, Slot::Ready(result));
            }
            _ => {
                inner.slots.remove(&index);
            }
        }
        waiters
    };
    waiters.into_iter().for_each(Waker::wake);
}

struct Wait<T> {
    inner: Rc<RefCell<Inner<T>>>,
    index: usize,
}

impl<T: Clone> Future for Wait<T> {
    type Output = Result<T, Fetch>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.inner.borrow_mut();
        match inner.slots.get_mut(&self.index) {
            Some(Slot::Ready(result)) => Poll::Ready(result.clone().map_err(Fetch::Failed)),
            Some(Slot::Pending { waiters, .. }) => {
                waiters.push(cx.waker().clone());
                Poll::Pending
            }
            None => Poll::Ready(Err(Fetch::Cancelled)),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Clone, Debug, PartialEq)]
    pub struct Fake(pub i32, pub i32);

    impl PageImage for Fake {
        fn size(&self) -> (i32, i32) {
            (self.0, self.1)
        }
    }

    /// Runs `f` to completion on a private main context, so tests can run in parallel.
    pub fn run<F: Future>(f: F) -> F::Output {
        let ctx = glib::MainContext::new();
        ctx.with_thread_default(|| ctx.block_on(f)).unwrap()
    }

    /// A session over fake pages: `sizes[i]` is page i's size, `None` fails to decode.
    /// The log records every decode.
    pub fn fake(sizes: Vec<Option<(i32, i32)>>) -> (Session<Fake>, Arc<Mutex<Vec<usize>>>) {
        let log = Arc::new(Mutex::new(Vec::new()));
        let l = log.clone();
        let count = sizes.len();
        let decoder: Decoder<Fake> = Arc::new(move |i, _| {
            l.lock().unwrap().push(i);
            sizes[i]
                .map(|(w, h)| Fake(w, h))
                .ok_or_else(|| format!("bad {i}"))
        });
        (Session::new(count, decoder), log)
    }

    /// Lets spawned decodes finish.
    pub async fn settle() {
        for _ in 0..50 {
            glib::timeout_future(std::time::Duration::from_millis(2)).await;
        }
    }

    #[test]
    fn decodes_and_caches() {
        let (s, log) = fake(vec![Some((1, 2)); 3]);
        run(async {
            assert_eq!(s.image(0, true).await, Ok(Fake(1, 2)));
            assert_eq!(s.image(0, true).await, Ok(Fake(1, 2)));
        });
        assert_eq!(*log.lock().unwrap(), [0], "decoded once");
    }

    #[test]
    fn keeps_only_pages_near_the_current_one() {
        let (s, _) = fake(vec![Some((1, 2)); 10]);
        run(async {
            for i in 0..6 {
                s.image(i, true).await.unwrap();
                s.prefetch(i);
                settle().await;
            }
        });
        assert_eq!(s.held(), [3, 4, 5, 6]);
    }

    #[test]
    fn prefetches_both_neighbours() {
        let (s, log) = fake(vec![Some((1, 2)); 5]);
        run(async {
            s.image(2, true).await.unwrap();
            s.prefetch(2);
            settle().await;
        });
        let mut decoded = log.lock().unwrap().clone();
        decoded.sort();
        assert_eq!(decoded, [1, 2, 3]);
    }

    #[test]
    fn a_page_paged_past_is_cancelled() {
        // Page 0's decode hangs until the test lets it go.
        let (release, gate) = std::sync::mpsc::channel::<()>();
        let gate = Mutex::new(gate);
        let decoder: Decoder<Fake> = Arc::new(move |i, _| {
            if i == 0 {
                let _ = gate.lock().unwrap().recv();
            }
            Ok(Fake(1, 2))
        });
        let s = Session::new(10, decoder);
        let result = run(async {
            let s2 = s.clone();
            let early = glib::MainContext::ref_thread_default()
                .spawn_local(async move { s2.image(0, true).await });
            glib::timeout_future(std::time::Duration::from_millis(20)).await;
            s.image(9, true).await.unwrap();
            early.await.unwrap()
        });
        release.send(()).unwrap();
        assert_eq!(result, Err(Fetch::Cancelled));
        assert_eq!(s.held(), [9]);
    }

    #[test]
    fn a_companion_fetch_does_not_move_the_window() {
        let (s, _) = fake(vec![Some((1, 2)); 10]);
        run(async {
            s.image(5, true).await.unwrap();
            s.image(6, false).await.unwrap();
            s.image(3, true).await.unwrap();
        });
        assert_eq!(s.held(), [3, 5]);
    }

    #[test]
    fn failures_are_reported() {
        let (s, _) = fake(vec![None]);
        let r = run(s.image(0, true));
        assert_eq!(r, Err(Fetch::Failed("bad 0".into())));
        assert!(matches!(run(s.image(1, true)), Err(Fetch::Failed(_))));
    }
}
