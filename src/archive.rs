//! The one place that knows about ZIP. Everything else sees entry paths and bytes.

use std::fmt;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;
use std::sync::Mutex;

/// Entries bigger than this are refused rather than inflated (`PROD-7`): no comic page
/// is anywhere near it, and a ZIP bomb is.
const MAX_ENTRY_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug, PartialEq)]
pub enum Format {
    Zip,
    Rar,
    Unrecognized,
}

/// What a file is from its first bytes (`READ-1`).
pub fn format_of(magic: &[u8]) -> Format {
    match magic {
        [b'R', b'a', b'r', b'!', ..] => Format::Rar,
        [b'P', b'K', 3 | 5 | 7, ..] if magic.len() >= 4 => Format::Zip,
        _ => Format::Unrecognized,
    }
}

/// Errors the user sees, with the texts from `READ-1`, `READ-3`…`READ-5`.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Rar(String),
    NotZip(String),
    NoPages(String),
    Open { name: String, reason: String },
    Undecodable(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Rar(name) => write!(
                f,
                "“{name}” is a RAR archive (a .cbr), not a .cbz. \
                 cmx reads .cbz only — convert it to .cbz to open it here."
            ),
            Error::NotZip(name) => {
                write!(f, "“{name}” isn’t a .cbz — it’s not a ZIP archive at all.")
            }
            Error::NoPages(name) => write!(f, "No pages found in “{name}”."),
            Error::Open { name, reason } => write!(f, "Couldn't open “{name}”: {reason}"),
            Error::Undecodable(path) => write!(f, "Couldn't decode the image “{path}”."),
        }
    }
}

pub fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

/// `io::Error` text without the "(os error 2)" tail.
fn reason(e: &dyn fmt::Display) -> String {
    let text = e.to_string();
    match text.rfind(" (os error ") {
        Some(i) => text[..i].to_owned(),
        None => text,
    }
}

/// An open archive, kept for as long as its comic is shown. The ZIP reader isn't
/// shareable, so reads take turns behind the lock.
pub struct Archive {
    zip: Mutex<zip::ZipArchive<File>>,
    /// Raw file entry paths, unfiltered and in archive order.
    pub entries: Vec<String>,
}

impl Archive {
    pub fn open(path: &Path) -> Result<Self, Error> {
        let name = file_name(path);
        let open_error = |e: &dyn fmt::Display| Error::Open {
            name: name.clone(),
            reason: reason(e),
        };
        let mut file = File::open(path).map_err(|e| open_error(&e))?;
        let mut magic = Vec::with_capacity(4);
        (&mut file)
            .take(4)
            .read_to_end(&mut magic)
            .map_err(|e| open_error(&e))?;
        match format_of(&magic) {
            Format::Zip => {}
            Format::Rar => return Err(Error::Rar(name)),
            Format::Unrecognized => return Err(Error::NotZip(name)),
        }
        file.rewind().map_err(|e| open_error(&e))?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| open_error(&e))?;
        let mut entries = Vec::with_capacity(zip.len());
        for i in 0..zip.len() {
            let entry = zip.by_index_raw(i).map_err(|e| open_error(&e))?;
            if entry.is_file() {
                entries.push(entry.name().map_err(|e| open_error(&e))?.into_owned());
            }
        }
        Ok(Self {
            zip: Mutex::new(zip),
            entries,
        })
    }

    /// The bytes of one entry. Safe to call from any thread.
    pub fn read(&self, entry: &str) -> Result<Vec<u8>, String> {
        let mut zip = self.zip.lock().unwrap_or_else(|e| e.into_inner());
        let file = zip.by_name(entry).map_err(|e| e.to_string())?;
        if file.size() > MAX_ENTRY_BYTES {
            return Err(format!("{entry} is too large"));
        }
        let mut data = Vec::with_capacity(file.size() as usize);
        file.take(MAX_ENTRY_BYTES + 1)
            .read_to_end(&mut data)
            .map_err(|e| e.to_string())?;
        if data.len() as u64 > MAX_ENTRY_BYTES {
            return Err(format!("{entry} is too large"));
        }
        Ok(data)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    pub fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    /// A stored (uncompressed) ZIP at a fresh temp path.
    pub fn write_zip(name: &str, entries: &[(&str, &[u8])]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cmx-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (entry, data) in entries {
            if entry.ends_with('/') {
                zip.add_directory(*entry, options).unwrap();
            } else {
                zip.start_file(*entry, options).unwrap();
                zip.write_all(data).unwrap();
            }
        }
        zip.finish().unwrap();
        path
    }

    #[test]
    fn detects_formats_by_magic() {
        assert_eq!(format_of(b"PK\x03\x04"), Format::Zip);
        assert_eq!(format_of(b"PK\x05\x06"), Format::Zip);
        assert_eq!(format_of(b"PK\x07\x08"), Format::Zip);
        assert_eq!(format_of(b"Rar!\x1a\x07"), Format::Rar);
        assert_eq!(format_of(b"PK\x01\x02"), Format::Unrecognized);
        assert_eq!(format_of(b"PK\x03"), Format::Unrecognized);
        assert_eq!(format_of(b""), Format::Unrecognized);
        assert_eq!(format_of(b"%PDF"), Format::Unrecognized);
    }

    #[test]
    fn mislabeled_files_explain_themselves() {
        let rar = Archive::open(&fixture("rar-disguised.cbz")).err().unwrap();
        assert_eq!(rar, Error::Rar("rar-disguised.cbz".into()));
        assert_eq!(
            rar.to_string(),
            "“rar-disguised.cbz” is a RAR archive (a .cbr), not a .cbz. \
             cmx reads .cbz only — convert it to .cbz to open it here."
        );
        let other = Archive::open(&fixture("not-an-archive.cbz")).err().unwrap();
        assert_eq!(
            other.to_string(),
            "“not-an-archive.cbz” isn’t a .cbz — it’s not a ZIP archive at all."
        );
    }

    #[test]
    fn open_failures_name_the_file() {
        let missing = Archive::open(Path::new("/nonexistent/gone.cbz"))
            .err()
            .unwrap();
        assert_eq!(
            missing.to_string(),
            "Couldn't open “gone.cbz”: No such file or directory"
        );
        let dir = std::env::temp_dir().join(format!("cmx-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let broken = dir.join("broken.cbz");
        std::fs::write(&broken, b"PK\x03\x04 then nothing useful").unwrap();
        let err = Archive::open(&broken).err().unwrap();
        assert!(
            matches!(&err, Error::Open { name, .. } if name == "broken.cbz"),
            "{err:?}"
        );
    }

    #[test]
    fn lists_file_entries_and_reads_them() {
        let archive = Archive::open(&fixture("sample.cbz")).unwrap();
        assert!(archive.entries.contains(&"page10.png".to_owned()));
        assert!(archive.entries.contains(&"ComicInfo.xml".to_owned()));
        assert!(archive.read("page1.png").unwrap().starts_with(b"\x89PNG"));
        assert!(archive.read("missing.png").is_err());

        let path = write_zip("dirs.cbz", &[("ch1/", b""), ("ch1/a.png", b"x")]);
        let archive = Archive::open(&path).unwrap();
        assert_eq!(archive.entries, ["ch1/a.png"]);
    }
}
