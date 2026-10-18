# Appearance

## Requirements

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `LOOK-1` | Display scale: every size in the spec and in the app's CSS is in logical px, so at scales 1, 1.25, 1.5 and 2 the window has the same layout in logical px, and icons and text are drawn at the display's full resolution, never upscaled. | done | `scripts/manual/scales.sh` (headless at each scale: screenshot; window size in logical px matches `[window]`) |

## Implementation notes

- At scale 1, GTK's default font rendering leaves 1px strokes in small text faint. If that matters, set `gtk-font-rendering` to manual with hinted metrics only when every monitor has scale 1 (see Junk Drawer's `theme::set_font_rendering`).
