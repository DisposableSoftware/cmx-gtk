//! Page bytes to images. Safe to call off the main thread.

use gtk::gdk_pixbuf::{Pixbuf, PixbufLoader};
use gtk::{gdk, glib, prelude::*};

/// The gdk-pixbuf loader for an entry's extension. The loader is chosen by name
/// because sniffing misses real files: libavif's loader signature only matches a
/// 32-byte `ftyp` box, and nixpkgs builds gdk-pixbuf without GIO sniffing.
fn loader_type(entry: &str) -> Option<&'static str> {
    let ext = entry.rsplit_once('.')?.1.to_ascii_lowercase();
    Some(match ext.as_str() {
        "jpg" | "jpeg" => "jpeg",
        "tif" | "tiff" => "tiff",
        "png" => "png",
        "gif" => "gif",
        "webp" => "webp",
        "avif" => "avif",
        "bmp" => "bmp",
        _ => return None,
    })
}

pub fn pixbuf(entry: &str, data: &[u8]) -> Option<Pixbuf> {
    let by_type = loader_type(entry).and_then(|t| PixbufLoader::with_type(t).ok());
    by_type
        .into_iter()
        .chain([PixbufLoader::new()])
        .find_map(|loader| {
            let ok = loader.write(data).is_ok();
            let closed = loader.close().is_ok();
            (ok && closed).then(|| loader.pixbuf()).flatten()
        })
}

pub fn texture(entry: &str, data: Vec<u8>) -> Option<gdk::Texture> {
    let bytes = glib::Bytes::from_owned(data);
    // GDK decodes PNG, JPEG and TIFF itself, faster than gdk-pixbuf.
    if let Ok(texture) = gdk::Texture::from_bytes(&bytes) {
        return Some(texture);
    }
    let p = pixbuf(entry, &bytes)?;
    let format = if p.has_alpha() {
        gdk::MemoryFormat::R8g8b8a8
    } else {
        gdk::MemoryFormat::R8g8b8
    };
    let texture = gdk::MemoryTexture::new(
        p.width(),
        p.height(),
        format,
        &p.read_pixel_bytes(),
        p.rowstride() as usize,
    );
    Some(texture.upcast())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_loader_from_the_extension() {
        assert_eq!(loader_type("a/b/001.JPG"), Some("jpeg"));
        assert_eq!(loader_type("x.tif"), Some("tiff"));
        assert_eq!(loader_type("x.avif"), Some("avif"));
        assert_eq!(loader_type("x.txt"), None);
        assert_eq!(loader_type("noext"), None);
    }

    #[test]
    fn decodes_with_or_without_a_matching_extension() {
        let png = include_bytes!("../tests/fixtures/pixel.png");
        assert!(pixbuf("p.png", png).is_some());
        assert!(pixbuf("p.jpg", png).is_some(), "falls back to sniffing");
        assert!(pixbuf("p.png", b"not an image").is_none());
        assert!(texture("p.png", png.to_vec()).is_some());
        assert!(texture("p.png", b"junk".to_vec()).is_none());
    }

    /// Only where the AVIF loader is installed, as in the package's wrapper.
    #[test]
    fn decodes_avif_when_the_loader_exists() {
        if PixbufLoader::with_type("avif").is_err() {
            eprintln!("skipping: no AVIF loader");
            return;
        }
        let archive =
            crate::archive::Archive::open(&crate::archive::tests::fixture("formats.cbz")).unwrap();
        let data = archive.read("2.avif").unwrap();
        assert!(pixbuf("2.avif", &data).is_some());
    }
}
