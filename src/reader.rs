//! A comic and the rules for moving through it, without any widgets.

use crate::archive::{self, Archive, Error};
use crate::metadata::{self, Metadata};
use crate::pages;
use crate::session::{Fetch, PageImage, Session};
use std::path::{Path, PathBuf};

pub struct Book {
    pub path: PathBuf,
    pub pages: Vec<String>,
    pub metadata: Option<Metadata>,
}

impl Book {
    /// `READ-14`: metadata's title, or the file name without its extension.
    pub fn display_title(&self) -> String {
        self.metadata
            .as_ref()
            .and_then(Metadata::display_title)
            .unwrap_or_else(|| stem(&self.path))
    }
}

fn stem(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| archive::file_name(path))
}

/// Opens the archive, derives the pages and reads the metadata. Blocking: run it
/// off the main thread.
pub fn open(path: &Path) -> Result<(Book, Archive), Error> {
    let archive = Archive::open(path)?;
    let pages = pages::pages(&archive.entries);
    if pages.is_empty() {
        return Err(Error::NoPages(archive::file_name(path)));
    }
    // Best effort: a missing or broken ComicInfo.xml never fails the open.
    let metadata = metadata::locate(&archive.entries)
        .and_then(|entry| archive.read(entry).ok())
        .and_then(|xml| metadata::parse(&xml));
    let book = Book {
        path: path.to_owned(),
        pages,
        metadata,
    };
    Ok((book, archive))
}

/// Where a saved page resumes, clamped to the page list (`READ-40`).
pub fn resume(saved: Option<usize>, count: usize) -> usize {
    saved.unwrap_or(0).min(count.saturating_sub(1))
}

/// `READ-20`. Each returns the page to go to, or `None` to do nothing.
pub fn next(index: usize, shown: usize, count: usize) -> Option<usize> {
    let target = index + shown;
    (target < count).then_some(target)
}

pub fn previous(index: usize, dual: bool) -> Option<usize> {
    (index > 0).then(|| index.saturating_sub(if dual { 2 } else { 1 }))
}

pub fn first(index: usize) -> Option<usize> {
    (index != 0).then_some(0)
}

pub fn last(index: usize, count: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    (index != last).then_some(last)
}

/// `WIN-7`: "3 / 42", or "3–4 / 42" with two pages shown.
pub fn indicator(index: usize, shown: usize, count: usize) -> String {
    let first = index + 1;
    if shown == 2 {
        format!("{first}–{} / {count}", first + 1)
    } else {
        format!("{first} / {count}")
    }
}

pub fn window_title(book: Option<(&Book, usize, usize)>) -> String {
    match book {
        Some((book, index, shown)) => format!(
            "{} · {}",
            book.display_title(),
            indicator(index, shown, book.pages.len())
        ),
        None => "cmx".into(),
    }
}

/// The page at `index` and, in dual-page mode, its partner (`READ-30`, `READ-32`).
/// The current page's shape is checked before the next one is fetched.
pub async fn spread<T: PageImage>(
    session: &Session<T>,
    index: usize,
    dual: bool,
) -> Result<(T, Option<T>), Fetch> {
    let page = session.image(index, true).await?;
    if !dual || page.is_wide() || index + 1 >= session.count() {
        return Ok((page, None));
    }
    let partner = match session.image(index + 1, false).await {
        Ok(next) if !next.is_wide() => Some(next),
        Ok(_) | Err(Fetch::Failed(_)) => None,
        Err(Fetch::Cancelled) => return Err(Fetch::Cancelled),
    };
    Ok((page, partner))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::tests::{fixture, write_zip};
    use crate::session::tests::{Fake, fake, run};

    const PNG: &[u8] = include_bytes!("../tests/fixtures/pixel.png");

    #[test]
    fn opens_the_sample() {
        let (book, archive) = open(&fixture("sample.cbz")).unwrap();
        assert_eq!(book.pages, ["page1.png", "page2.png", "page10.png"]);
        assert!(book.metadata.is_some());
        assert!(archive.read(&book.pages[0]).is_ok());
    }

    #[test]
    fn an_archive_without_pages_is_an_error() {
        let path = write_zip(
            "nopages.cbz",
            &[("ComicInfo.xml", b"<ComicInfo/>"), ("a.txt", b"x")],
        );
        let err = open(&path).err().unwrap();
        assert_eq!(err.to_string(), "No pages found in “nopages.cbz”.");
    }

    #[test]
    fn broken_metadata_never_fails_the_open() {
        let path = write_zip("badxml.cbz", &[("ComicInfo.xml", b"<<<"), ("1.png", PNG)]);
        let (book, _) = open(&path).unwrap();
        assert!(book.metadata.is_none());
        assert_eq!(book.display_title(), "badxml");
    }

    #[test]
    fn page_count_metadata_is_not_used_for_navigation() {
        let path = write_zip(
            "liar.cbz",
            &[
                (
                    "ComicInfo.xml",
                    b"<ComicInfo><PageCount>99</PageCount></ComicInfo>",
                ),
                ("1.png", PNG),
                ("2.png", PNG),
            ],
        );
        let (book, _) = open(&path).unwrap();
        assert_eq!(book.metadata.as_ref().unwrap().page_count, Some(99));
        assert_eq!(book.pages.len(), 2);
        assert_eq!(last(0, book.pages.len()), Some(1));
    }

    #[test]
    fn navigation_stops_at_both_ends() {
        assert_eq!(next(0, 1, 3), Some(1));
        assert_eq!(next(2, 1, 3), None);
        assert_eq!(next(1, 2, 4), Some(3));
        assert_eq!(next(2, 2, 4), None);
        assert_eq!(previous(0, false), None);
        assert_eq!(previous(0, true), None);
        assert_eq!(previous(3, false), Some(2));
        assert_eq!(previous(3, true), Some(1));
        // From an odd index in dual mode, page 1 stays reachable.
        assert_eq!(previous(1, true), Some(0));
        assert_eq!(first(0), None);
        assert_eq!(first(4), Some(0));
        assert_eq!(last(4, 5), None);
        assert_eq!(last(0, 5), Some(4));
        assert_eq!(resume(Some(10), 5), 4);
        assert_eq!(resume(None, 5), 0);
    }

    #[test]
    fn titles_and_indicators() {
        assert_eq!(indicator(2, 1, 42), "3 / 42");
        assert_eq!(indicator(2, 2, 42), "3–4 / 42");
        let book = Book {
            path: "/c/Some Comic.cbz".into(),
            pages: vec!["a".into(); 6],
            metadata: None,
        };
        assert_eq!(window_title(Some((&book, 0, 1))), "Some Comic · 1 / 6");
        assert_eq!(window_title(None), "cmx");
    }

    /// The fixture walk from the spec: 6 portrait pages but page 3 is wide.
    #[test]
    fn dual_page_never_pairs_a_wide_page() {
        let mut sizes = vec![Some((100, 150)); 6];
        sizes[2] = Some((300, 150));
        let (s, _) = fake(sizes);
        let walk = run(async {
            let mut seen = Vec::new();
            let mut index = 0;
            loop {
                let (_, partner) = spread(&s, index, true).await.unwrap();
                let shown = if partner.is_some() { 2 } else { 1 };
                seen.push(indicator(index, shown, 6));
                match next(index, shown, 6) {
                    Some(n) => index = n,
                    None => break seen,
                }
            }
        });
        assert_eq!(walk, ["1–2 / 6", "3 / 6", "4–5 / 6", "6 / 6"]);
    }

    #[test]
    fn a_wide_page_is_shown_without_fetching_its_neighbour() {
        let (s, log) = fake(vec![Some((300, 150)), Some((100, 150))]);
        let (_, partner) = run(spread(&s, 0, true)).unwrap();
        assert_eq!(partner, None);
        assert_eq!(*log.lock().unwrap(), [0]);
        let (s, log) = fake(vec![Some((100, 150)), Some((100, 150))]);
        let (_, partner) = run(spread(&s, 0, true)).unwrap();
        assert_eq!(partner, Some(Fake(100, 150)));
        assert_eq!(log.lock().unwrap().len(), 2);
    }

    #[test]
    fn an_undecodable_page_is_an_error_but_its_neighbours_work() {
        let (s, _) = fake(vec![Some((1, 2)), None, Some((1, 2))]);
        assert!(run(spread(&s, 0, false)).is_ok());
        assert_eq!(
            run(spread(&s, 1, false)),
            Err(Fetch::Failed("bad 1".into()))
        );
        assert!(run(spread(&s, 2, false)).is_ok());
        // A broken partner just leaves the page alone.
        let (_, partner) = run(spread(&s, 0, true)).unwrap();
        assert_eq!(partner, None);
    }
}
