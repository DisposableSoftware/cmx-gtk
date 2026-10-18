# Reader

How a `.cbz` becomes pages, metadata and saved progress. These rules are where real bugs come from.

## Archive

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `READ-1` | A file's type comes from its first 4 bytes, not its extension. `PK` followed by byte 03, 05 or 07 is a ZIP, and is read. `Rar!` gives the error "“NAME” is a RAR archive (a .cbr), not a .cbz. cmx reads .cbz only — convert it to .cbz to open it here." Anything else gives "“NAME” isn’t a .cbz — it’s not a ZIP archive at all." NAME is the file name. | done | `archive::tests` (fixtures `rar-disguised.cbz`, `not-an-archive.cbz`) |
| `READ-2` | The page list is the archive's file entries, filtered and then sorted. Directories are dropped, and so is any path with a component that starts with `.` or is `__MACOSX`, and any file named `Thumbs.db` (any case). Only the extensions `jpg` `jpeg` `png` `gif` `webp` `avif` `bmp` `tif` `tiff` (any case) are kept. The list is sorted by full entry path in natural order: digit runs compare as numbers and other text compares case-insensitively, so `page2.jpg` comes before `page10.jpg`. The order of entries in the archive is never used. | done | `pages::tests` |
| `READ-3` | An archive with no pages gives the error "No pages found in “NAME”.", and no comic opens. | done | `reader::tests` |
| `READ-4` | Any other failure to open (unreadable file, corrupt ZIP) gives "Couldn't open “NAME”: REASON". | done | `reader::tests` |
| `READ-5` | A page that can't be decoded gives "Couldn't decode the image “PATH”." PATH is the entry path. The comic stays open, and the other pages still work. | done | `reader::tests` |

## Metadata

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `READ-10` | `ComicInfo.xml` (ComicRack schema) is read from the archive root, name compared case-insensitively. When the root has none, a copy in any folder is used. A missing file is normal and never an error. | done | `metadata::tests` |
| `READ-11` | Parsing is lenient: only the root element's direct children are read, and unknown elements and nested containers (such as `<Pages>`) are ignored. When the XML is malformed, the fields read before the error are kept. With no fields, the comic has no metadata. Metadata never fails an open. | done | `metadata::tests` (full, partial, absent, malformed, truncated, nested) |
| `READ-12` | The fields read are `Series`, `Number`, `Count`, `Volume`, `Title`, `Summary`, `Writer`, `Penciller`, `Inker`, `Colorist`, `Letterer`, `CoverArtist`, `Publisher`, `Year`, `Month`, `Day`, `PageCount`, `LanguageISO`, `Genre` and `Web`. Values are trimmed, and empty values are left out. `Number` is text ("7.1", "Annual 1"). A numeric field (`Count`, `Volume`, `Year`, `Month`, `Day`, `PageCount`) that isn't an integer is left out. | done | `metadata::tests` |
| `READ-13` | `PageCount` is only shown (`WIN-13`). Navigation always counts the real page list. | done | `reader::tests` |
| `READ-14` | The display title is `Series #Number — Title`, built from the parts that exist: `Series #Number`, `Series`, `#Number`, `Series — Title`, or `Title`. When none exist, the display title is the file name without its extension. | done | `metadata::tests` |

## Navigation

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `READ-20` | `next-page` moves past every page on screen: 2 when two pages are shown, otherwise 1. `previous-page` moves back 2 in dual-page mode and 1 otherwise, but never before the first page. `first-page` and `last-page` go to the first and last page. A move past either end does nothing: it never wraps and never beeps. | done | `reader::tests` |
| `READ-21` | Paging feels instant. The archive stays open while the comic is shown. Pages are decoded off the main thread, and the pages before and after the current one are decoded in advance. Only pages within 2 of the current page stay decoded, and a decode the user has paged past is dropped. A decode that finishes late never replaces a newer page on screen. | done | `reader::tests` (cache window, late result), manual (`scripts/manual/paging.sh`) |

## Dual-page layout

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `READ-30` | `toggle-dual-page` turns dual-page layout on or off for that window. When it is on, the current page is shown on the left and the next page on the right. A page is never paired when it or the next page is landscape (wider than tall); it is shown alone. Pairing starts at the current page, so there is no special case for the cover. | done | `spread::tests`, manual (`scripts/manual/paging.sh`: on a 6-page fixture whose page 3 is wide, `1 / 6` → `d` → `1–2 / 6` → `3 / 6` → `4–5 / 6` → `6 / 6`) |
| `READ-31` | Dual-page mode isn't saved. Every window starts in single-page mode. Toggling it redraws the window in place and doesn't resize it. | done | manual (`scripts/manual/paging.sh`) |
| `READ-32` | To decide whether to pair, the shape of the current page is checked first, so a landscape page is shown without waiting for the next page to decode. Fetching the right-hand page of a spread doesn't move the window of decoded pages (`READ-21`). | done | `reader::tests` |

## Progress and recent files

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `READ-40` | The current page of each comic is saved every time it changes, keyed by the comic's absolute path, in `$XDG_STATE_HOME/cmx/state.ini`. The file is written atomically. Opening the comic again starts at the saved page, clamped to the page list. A file that has moved has no saved page, and that is not an error. | done | `progress::tests` |
| `READ-41` | The path of the last comic that opened successfully is saved in the same file. When cmx starts with no files, its first window opens that comic if the file still exists. Otherwise the window shows the placeholder (`WIN-5`). | done | `progress::tests`, manual (`scripts/manual/open.sh`) |
| `READ-42` | A comic that opens successfully is added to the recently used files list (`recently-used.xbel`, through GtkRecentManager), so file choosers list it. A file that fails to open is never added. | done | manual (`scripts/manual/open.sh`: `recently-used.xbel` in the temp data dir) |

## Implementation notes

- RARs renamed to `.cbz` are common, so content always wins over the extension. This is the same rule as `cbz-tool`'s `archive::is_rar_file`.
- Keep the ZIP reader behind `archive.rs`, so the crate can be swapped without touching anything else.
- `state.ini` is runtime state, not a setting, so it lives in `XDG_STATE_HOME` and never in `config.ini`.
- Decoding picks the gdk-pixbuf loader by the entry's extension (`decode.rs`) and only then sniffs. nixpkgs builds gdk-pixbuf without GIO sniffing, and libavif's loader signature only matches a 32-byte `ftyp` box, so sniffing alone rejected ordinary AVIF pages (a 28-byte box) as unrecognised.
