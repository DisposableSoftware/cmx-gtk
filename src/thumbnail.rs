//! `cmx --thumbnail` (`INT-51`): the cover as a PNG, for file managers. Uses
//! gdk-pixbuf only, so it needs no display and never starts GTK.

use crate::archive::{Archive, Error, file_name};
use crate::{decode, pages};
use gtk::gdk_pixbuf::InterpType;
use std::path::Path;

pub fn run(size: i32, input: &Path, output: &Path) -> Result<(), String> {
    let archive = Archive::open(input).map_err(|e| e.to_string())?;
    let pages = pages::pages(&archive.entries);
    let first = pages
        .first()
        .ok_or_else(|| Error::NoPages(file_name(input)).to_string())?;
    let undecodable = || Error::Undecodable(first.clone()).to_string();
    let data = archive.read(first).map_err(|_| undecodable())?;
    let pixbuf = decode::pixbuf(first, &data).ok_or_else(undecodable)?;
    let (w, h) = (pixbuf.width(), pixbuf.height());
    let scale = (f64::from(size) / f64::from(w.max(h))).min(1.0);
    let pixbuf = if scale < 1.0 {
        let scaled = |n: i32| ((f64::from(n) * scale).round() as i32).max(1);
        pixbuf
            .scale_simple(scaled(w), scaled(h), InterpType::Bilinear)
            .ok_or_else(undecodable)?
    } else {
        pixbuf
    };
    pixbuf
        .savev(output, "png", &[])
        .map_err(|e| format!("{}: {}", output.display(), e.message()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::tests::{fixture, write_zip};
    use gtk::gdk_pixbuf::Pixbuf;

    fn out(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("cmx-test-thumb-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        let _ = std::fs::remove_file(&path);
        path
    }

    fn png(w: i32, h: i32) -> Vec<u8> {
        let p = Pixbuf::new(gtk::gdk_pixbuf::Colorspace::Rgb, false, 8, w, h).unwrap();
        p.fill(0x3050c0ff);
        p.save_to_bufferv("png", &[]).unwrap()
    }

    #[test]
    fn writes_the_first_page_scaled_to_fit() {
        let big = png(300, 600);
        let path = write_zip("thumb.cbz", &[("p10.png", &png(50, 50)), ("p2.png", &big)]);
        let output = out("thumb.png");
        run(128, &path, &output).unwrap();
        let thumb = Pixbuf::from_file(&output).unwrap();
        assert_eq!((thumb.width(), thumb.height()), (64, 128), "p2 comes first");

        let output = out("small.png");
        let path = write_zip("small.cbz", &[("1.png", &png(40, 20))]);
        run(256, &path, &output).unwrap();
        let thumb = Pixbuf::from_file(&output).unwrap();
        assert_eq!((thumb.width(), thumb.height()), (40, 20), "never scaled up");
    }

    #[test]
    fn fails_without_writing_for_bad_input() {
        let output = out("bad.png");
        for input in [
            fixture("rar-disguised.cbz"),
            fixture("not-an-archive.cbz"),
            write_zip("empty.cbz", &[("a.txt", b"x")]),
            write_zip("garbage.cbz", &[("1.png", b"not a png")]),
        ] {
            assert!(run(128, &input, &output).is_err(), "{}", input.display());
            assert!(!output.exists());
        }
    }
}
