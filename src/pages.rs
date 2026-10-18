use std::cmp::Ordering;

pub const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "avif", "bmp", "tif", "tiff",
];

/// The page list (`READ-2`): image entries without the junk, in natural order. The
/// archive's own entry order is never used.
pub fn pages(entries: &[String]) -> Vec<String> {
    let mut pages: Vec<String> = entries.iter().filter(|e| is_page(e)).cloned().collect();
    pages.sort_by(|a, b| natural_cmp(a, b));
    pages
}

fn is_page(path: &str) -> bool {
    if path.ends_with('/') {
        return false;
    }
    if path
        .split('/')
        .any(|c| c.starts_with('.') || c == "__MACOSX")
    {
        return false;
    }
    let name = path.rsplit('/').next().unwrap_or(path);
    if name.eq_ignore_ascii_case("thumbs.db") {
        return false;
    }
    name.rsplit_once('.').is_some_and(|(_, ext)| {
        IMAGE_EXTENSIONS
            .iter()
            .any(|known| ext.eq_ignore_ascii_case(known))
    })
}

/// Digit runs compare as numbers, everything else case-insensitively; ties fall back
/// to a plain comparison so the order is total.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return a.cmp(b),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(c), Some(d)) if c.is_ascii_digit() && d.is_ascii_digit() => {
                let n = digits(&mut x);
                let m = digits(&mut y);
                let (n, m) = (n.trim_start_matches('0'), m.trim_start_matches('0'));
                let ord = n.len().cmp(&m.len()).then_with(|| n.cmp(m));
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            (Some(c), Some(d)) => {
                let ord = c.to_lowercase().cmp(d.to_lowercase());
                if ord != Ordering::Equal {
                    return ord;
                }
                x.next();
                y.next();
            }
        }
    }
}

fn digits(it: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut out = String::new();
    while let Some(c) = it.next_if(char::is_ascii_digit) {
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(entries: &[&str]) -> Vec<String> {
        pages(&entries.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn sorts_numerically_not_lexicographically() {
        assert_eq!(
            list(&["page10.jpg", "page2.jpg", "page1.jpg"]),
            ["page1.jpg", "page2.jpg", "page10.jpg"]
        );
        assert_eq!(
            list(&["b/1.png", "A/2.png", "a/10.png"]),
            ["A/2.png", "a/10.png", "b/1.png"]
        );
        assert_eq!(list(&["p007.png", "p10.png"]), ["p007.png", "p10.png"]);
    }

    #[test]
    fn filters_junk_entries() {
        assert_eq!(
            list(&[
                "page1.png",
                "ComicInfo.xml",
                "Thumbs.db",
                "THUMBS.DB",
                ".DS_Store",
                "__MACOSX/._page1.png",
                "notes.txt",
                "folder/",
                ".hidden.png",
                "noextension",
                "page2.png",
            ]),
            ["page1.png", "page2.png"]
        );
    }

    #[test]
    fn keeps_nested_images_and_drops_hidden_directories() {
        assert_eq!(
            list(&[
                "ch1/p1.jpg",
                ".cache/p0.jpg",
                "ch1/.thumbs/p1.jpg",
                "ch2/p1.jpg"
            ]),
            ["ch1/p1.jpg", "ch2/p1.jpg"]
        );
    }

    #[test]
    fn accepts_every_documented_extension_in_any_case() {
        let entries: Vec<String> = IMAGE_EXTENSIONS
            .iter()
            .enumerate()
            .map(|(i, e)| format!("{i}.{}", e.to_uppercase()))
            .collect();
        assert_eq!(pages(&entries).len(), IMAGE_EXTENSIONS.len());
    }

    #[test]
    fn empty_for_an_imageless_archive() {
        assert!(list(&["ComicInfo.xml", "readme.txt"]).is_empty());
    }
}
